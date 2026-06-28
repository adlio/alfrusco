use alfrusco::simulator::{ActionResult, Screen, Simulator};
use alfrusco::{Item, Runnable, Workflow};

/// A test workflow that produces items with specific valid/autocomplete settings.
struct TestMenuWorkflow {
    /// If true, produces items with valid:false + autocomplete (loopback pattern).
    autocomplete_mode: bool,
}

impl Runnable for TestMenuWorkflow {
    type Error = alfrusco::Error;

    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        if self.autocomplete_mode {
            // Loopback items: valid:false + autocomplete
            wf.append_item(
                Item::new("Fruits")
                    .subtitle("3 items")
                    .arg("fruits")
                    .valid(false)
                    .autocomplete("fruits"),
            );
        } else {
            // Normal actionable items
            wf.append_item(
                Item::new("Apple")
                    .subtitle("A fruit")
                    .arg("https://en.wikipedia.org/wiki/Apple")
                    .valid(true),
            );
        }
        Ok(())
    }
}

// --- Good plist: action routes through conditional to sub-filter (DrilledIn) ---

#[test]
fn action_routes_through_conditional_to_script_filter() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .source_filter("SF-MAIN-001");

    let screen = sim
        .run_in_process(TestMenuWorkflow {
            autocomplete_mode: false,
        })
        .unwrap();

    // SF-MAIN-001 → COND-001 → SF-SUB-001 (DrilledIn)
    let result = screen.action(0).unwrap();
    assert_eq!(
        result,
        ActionResult::DrilledIn {
            target_uid: "SF-SUB-001".to_string()
        }
    );
    result.assert_drills_in();
}

// --- Good plist: sub-filter items route to OpenUrl ---

#[test]
fn action_from_sub_filter_routes_to_open_url() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .source_filter("SF-SUB-001");

    let screen = sim
        .run_in_process(TestMenuWorkflow {
            autocomplete_mode: false,
        })
        .unwrap();

    // SF-SUB-001 → ACTION-URL-001 (OpenedUrl)
    let result = screen.action(0).unwrap();
    assert_eq!(
        result,
        ActionResult::OpenedUrl {
            url_template: "{query}".to_string()
        }
    );
    result.assert_opens_url();
}

// --- Autocomplete loopback (valid:false + autocomplete) ---

#[test]
fn action_autocomplete_loopback() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .source_filter("SF-MAIN-001");

    let screen = sim
        .run_in_process(TestMenuWorkflow {
            autocomplete_mode: true,
        })
        .unwrap();

    let result = screen.action(0).unwrap();
    assert_eq!(
        result,
        ActionResult::TypedAutocomplete {
            text: "fruits".to_string()
        }
    );
    result.assert_autocompletes();
}

// --- Broken plist: dangling connection → DeadEnd ---

#[test]
fn action_dead_end_on_broken_plist() {
    // Create a screen with the broken plist's graph context
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_broken_workflow").unwrap();

    let screen = sim
        .run_in_process(TestMenuWorkflow {
            autocomplete_mode: false,
        })
        .unwrap();

    // SF-MAIN-001 → NONEXISTENT-UID-999 (dangling) → DeadEnd
    let result = screen.action(0).unwrap();
    assert!(result.is_dead_end());
}

// --- Screen without context returns None ---

#[test]
fn action_returns_none_without_context() {
    let json = r#"{"items":[{"title":"Test","valid":true}]}"#;
    let screen = Screen::from_json(json).unwrap();
    assert!(screen.action(0).is_none());
}

// --- action_first convenience ---

#[test]
fn action_first_on_empty_screen_returns_none() {
    let json = r#"{"items":[]}"#;
    let screen = Screen::from_json(json).unwrap();
    assert!(screen.action_first().is_none());
}

// --- Subprocess invocation with action routing ---

#[test]
fn action_routing_via_subprocess_top_level() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary("target/debug/examples/menu")
        .source_filter("SF-MAIN-001");

    let screen = sim.invoke(&[]).unwrap();
    // Top-level items have valid:true + autocomplete → autocomplete takes priority only if valid:false
    // These items are valid:true, so they go through graph routing → DrilledIn
    let result = screen.action(0).unwrap();
    assert_eq!(
        result,
        ActionResult::DrilledIn {
            target_uid: "SF-SUB-001".to_string()
        }
    );
}

#[test]
fn action_routing_via_subprocess_sub_level() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary("target/debug/examples/menu")
        .source_filter("SF-SUB-001");

    let screen = sim.invoke(&["fruits"]).unwrap();
    // Sub-level items route to Open URL
    let result = screen.action(0).unwrap();
    result.assert_opens_url();
}
