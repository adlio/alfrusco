//! Integration tests for the simulator harness (S3).
//!
//! Verifies that both in-process and subprocess invocation produce parsed items
//! from a workflow directory's real `info.plist`.

use alfrusco::simulator::Simulator;
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

/// A minimal test workflow that produces a fixed set of items.
struct TestWorkflow {
    query: String,
}

impl Runnable for TestWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        if self.query.is_empty() {
            workflow.append_item(
                Item::new("No Query")
                    .subtitle("No input provided")
                    .valid(false),
            );
        } else {
            workflow.append_item(
                Item::new(format!("Result for: {}", self.query))
                    .subtitle("From in-process test")
                    .arg(&self.query)
                    .valid(true),
            );
        }
        Ok(())
    }
}

#[test]
fn simulator_for_workflow_dir_parses_plist() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    assert_eq!(sim.bundleid(), "com.example.alfrusco.menu");
}

#[test]
fn simulator_for_workflow_dir_errors_on_missing_dir() {
    let result = Simulator::for_workflow_dir("/nonexistent/path");
    assert!(result.is_err());
}

#[test]
fn simulator_run_in_process_renders_items() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    let wf = TestWorkflow {
        query: "hello".to_string(),
    };
    let screen = sim.run_in_process(wf).unwrap();

    assert_eq!(screen.len(), 1);
    assert_eq!(screen.items()[0].title(), "Result for: hello");
    assert_eq!(screen.items()[0].arg(), Some("hello"));
    assert!(screen.items()[0].is_valid());
}

#[test]
fn simulator_run_in_process_empty_query() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    let wf = TestWorkflow {
        query: String::new(),
    };
    let screen = sim.run_in_process(wf).unwrap();

    assert_eq!(screen.len(), 1);
    assert_eq!(screen.items()[0].title(), "No Query");
    assert_eq!(screen.items()[0].subtitle(), Some("No input provided"));
    assert!(!screen.items()[0].is_valid());
}

#[test]
fn simulator_invoke_subprocess_menu_top_level() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(example_binary("menu"));

    let screen = sim.invoke(&[]).unwrap();
    assert_eq!(screen.len(), 3);
    assert_eq!(screen.items()[0].title(), "Fruits");
    assert_eq!(screen.items()[0].variable("category"), Some("fruits"));
    assert_eq!(screen.items()[0].autocomplete(), Some("fruits"));
}

#[test]
fn simulator_invoke_subprocess_menu_drill_in() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(example_binary("menu"));

    let screen = sim.invoke(&["fruits"]).unwrap();
    assert_eq!(screen.len(), 3);
    assert_eq!(screen.items()[0].title(), "Apple");
    assert_eq!(
        screen.items()[0].arg(),
        Some("https://en.wikipedia.org/wiki/Apple")
    );
    assert!(screen.items()[0].is_valid());
}

#[test]
fn simulator_invoke_errors_on_no_binary() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    let result = sim.invoke(&["test"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("no binary path set"));
}

#[test]
fn simulator_invoke_errors_on_missing_binary() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary("/nonexistent/binary");

    let result = sim.invoke(&["test"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("binary not found"));
}

#[test]
fn simulator_dir_overrides_work() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("my_cache");
    let data = tmp.path().join("my_data");

    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .cache_dir(&cache)
        .data_dir(&data);

    let wf = TestWorkflow {
        query: "test".to_string(),
    };
    let screen = sim.run_in_process(wf).unwrap();
    assert_eq!(screen.len(), 1);

    // Verify the directories were created
    assert!(cache.exists());
    assert!(data.exists());
}

#[test]
fn screen_assert_renders_passes_with_items() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    let wf = TestWorkflow {
        query: "x".to_string(),
    };
    let screen = sim.run_in_process(wf).unwrap();
    screen.assert_renders(); // Should not panic
}

#[test]
#[should_panic(expected = "Expected screen to render items")]
fn screen_assert_renders_panics_when_empty() {
    use alfrusco::simulator::Screen;
    let screen = Screen::from_json(r#"{"items":[]}"#).unwrap();
    screen.assert_renders();
}
