//! End-to-end behavioural tests demonstrating the workflow testing API (S5).
//!
//! These tests show how a workflow author writes navigation tests against
//! their real `info.plist` using the `Simulator` + `Screen` + `ActionResult` API.

use alfrusco::simulator::{ActionResult, Simulator, WorkflowGraph};
use alfrusco::{Item, Runnable, Workflow};

/// Returns the path to a built example binary, robust to any target directory.
fn example_binary(name: &str) -> String {
    let output = std::process::Command::new("cargo")
        .args(["build", "--example", name, "--message-format=json"])
        .output()
        .expect("failed to run cargo build");
    assert!(
        output.status.success(),
        "cargo build --example {name} failed"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in stdout.lines() {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
            if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-artifact")
                && msg
                    .get("target")
                    .and_then(|t| t.get("name"))
                    .and_then(|n| n.as_str())
                    == Some(name)
            {
                if let Some(exe) = msg.get("executable").and_then(|e| e.as_str()) {
                    return exe.to_string();
                }
            }
        }
    }
    panic!("could not find executable path for example '{name}' in cargo output");
}

// ---------------------------------------------------------------------------
// Reusable test workflow matching the menu example's logic
// ---------------------------------------------------------------------------

struct MenuWorkflow {
    category: Option<String>,
}

impl Runnable for MenuWorkflow {
    type Error = alfrusco::Error;

    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        match self.category.as_deref() {
            Some("fruits") => {
                for (title, url) in [
                    ("Apple", "https://en.wikipedia.org/wiki/Apple"),
                    ("Banana", "https://en.wikipedia.org/wiki/Banana"),
                    ("Cherry", "https://en.wikipedia.org/wiki/Cherry"),
                ] {
                    wf.append_item(
                        Item::new(title)
                            .subtitle(format!("Open {url}"))
                            .arg(url)
                            .valid(true),
                    );
                }
            }
            _ => {
                for (name, count) in [("fruits", 3), ("colors", 3), ("planets", 3)] {
                    wf.append_item(
                        Item::new(capitalize(name))
                            .subtitle(format!("{count} items"))
                            .arg(name)
                            .var("category", name)
                            .autocomplete(name)
                            .valid(true),
                    );
                }
            }
        }
        Ok(())
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ---------------------------------------------------------------------------
// End-to-end navigation walk: top-level → drill-in → URL action
// ---------------------------------------------------------------------------

/// Demonstrates the full behavioural test pattern a workflow author would write.
///
/// This test walks the menu workflow's navigation graph end-to-end:
/// 1. Render the top-level screen
/// 2. Action the first item → should drill into the sub-filter
/// 3. Render the sub-level screen (simulating the drill-in)
/// 4. Action a sub-level item → should open a URL
#[test]
fn end_to_end_menu_navigation_walk() {
    // Step 1: Create simulator from the workflow directory
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .source_filter("SF-MAIN-001");

    // Step 2: Render the top-level screen (no category selected)
    let top_screen = sim.run_in_process(MenuWorkflow { category: None }).unwrap();

    top_screen.assert_renders();
    assert_eq!(top_screen.len(), 3);
    assert_eq!(top_screen.items()[0].title(), "Fruits");
    assert_eq!(top_screen.items()[0].variable("category"), Some("fruits"));

    // Step 3: Action the first item — should drill in through the conditional
    let action = top_screen.action(0).unwrap();
    assert_eq!(
        action,
        ActionResult::DrilledIn {
            target_uid: "SF-SUB-001".to_string()
        }
    );
    action.assert_drills_in();

    // Step 4: Simulate the drill-in by rendering the sub-level
    let sub_sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .source_filter("SF-SUB-001");

    let sub_screen = sub_sim
        .run_in_process(MenuWorkflow {
            category: Some("fruits".to_string()),
        })
        .unwrap();

    sub_screen.assert_renders();
    assert_eq!(sub_screen.len(), 3);
    assert_eq!(sub_screen.items()[0].title(), "Apple");

    // Step 5: Action a sub-level item — should open a URL
    let sub_action = sub_screen.action(0).unwrap();
    assert_eq!(
        sub_action,
        ActionResult::OpenedUrl {
            url_template: "{query}".to_string()
        }
    );
    sub_action.assert_opens_url();
}

/// Tests that a broken workflow graph produces DeadEnd for actions.
#[test]
fn broken_graph_produces_dead_end() {
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_broken_workflow").unwrap();

    let screen = sim.run_in_process(MenuWorkflow { category: None }).unwrap();

    let action = screen.action(0).unwrap();
    assert!(action.is_dead_end());
}

/// Tests autocomplete loopback detection (valid:false + autocomplete).
#[test]
fn autocomplete_items_produce_typed_autocomplete() {
    struct AutocompleteWorkflow;
    impl Runnable for AutocompleteWorkflow {
        type Error = alfrusco::Error;
        fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
            wf.append_item(
                Item::new("Navigate to Fruits")
                    .arg("fruits")
                    .valid(false)
                    .autocomplete("fruits"),
            );
            Ok(())
        }
    }

    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    let screen = sim.run_in_process(AutocompleteWorkflow).unwrap();

    let action = screen.action_first().unwrap();
    assert_eq!(
        action,
        ActionResult::TypedAutocomplete {
            text: "fruits".to_string()
        }
    );
    assert_eq!(action.assert_autocompletes(), "fruits");
}

/// Tests graph auditing on a well-formed workflow (no defects).
#[test]
fn audit_clean_graph_has_no_diagnostics() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    let diagnostics = graph.audit_navigation(&["menu"]);
    assert!(
        diagnostics.is_empty(),
        "expected no defects, got: {diagnostics:?}"
    );
}

/// Tests graph auditing on a broken workflow (detects issues).
#[test]
fn audit_broken_graph_detects_defects() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_broken_workflow/info.plist").unwrap();
    let diagnostics = graph.audit_navigation(&["menu"]);
    assert!(
        !diagnostics.is_empty(),
        "expected defects in broken workflow"
    );
}

/// Tests that subprocess invocation works for the end-to-end walk.
#[test]
fn subprocess_end_to_end_walk() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(example_binary("menu"))
        .source_filter("SF-MAIN-001");

    // Top-level via subprocess
    let screen = sim.invoke(&[]).unwrap();
    screen.assert_renders();
    assert_eq!(screen.len(), 3);

    // Action first item → DrilledIn
    let action = screen.action(0).unwrap();
    action.assert_drills_in();

    // Drill into fruits via subprocess
    let sub_sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(example_binary("menu"))
        .source_filter("SF-SUB-001");

    let sub_screen = sub_sim.invoke(&["fruits"]).unwrap();
    sub_screen.assert_renders();
    assert_eq!(sub_screen.items()[0].title(), "Apple");

    // Action sub item → OpenedUrl
    let sub_action = sub_screen.action(0).unwrap();
    sub_action.assert_opens_url();
}
