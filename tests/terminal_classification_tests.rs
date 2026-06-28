//! R3 acceptance matrix: three-fixture audit comparison.
//!
//! The dynamic audit ERRORs ONLY on dead-ends. This test proves the
//! complete taxonomy across three fixtures:
//!
//! 1. External-Trigger drill-in → audit CLEAN (DrilledIn is OK)
//! 2. Act-and-exit (nav→RanScript) → audit CLEAN (RanScript is OK)
//! 3. Dangling-loopback (matched branch → nonexistent UID) → audit ERROR (DeadEnd)

use alfrusco::simulator::{Severity, Simulator, WorkflowGraph};
use std::sync::Arc;

/// Build the menu example binary once (shared across tests).
fn menu_binary() -> &'static str {
    static BUILD: std::sync::Once = std::sync::Once::new();
    BUILD.call_once(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "--example", "menu"])
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build --example menu failed");
    });
    "target/debug/examples/menu"
}

/// Fixture (i): External-Trigger drill-in — audit MUST be clean.
#[test]
fn matrix_external_trigger_audit_clean() {
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_external_trigger_workflow")
        .unwrap()
        .binary(menu_binary());

    let diagnostics = sim.dynamic_audit().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "fixture (i) external-trigger drill-in should be clean, got: {errors:#?}"
    );
}

/// Fixture (i): Verify the drill-in route through external trigger.
#[test]
fn matrix_external_trigger_drills_in() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_external_trigger_workflow/info.plist")
            .unwrap();

    let json = r#"{"items":[{"title":"Fruits","arg":"loopback","valid":true}]}"#;
    let screen = alfrusco::simulator::Screen::from_json(json).unwrap();
    let screen = screen.with_context(Arc::new(graph), "SF-MAIN-001".to_string());
    let action = screen.action(0).unwrap();
    assert_eq!(action.assert_drills_in(), "SF-SUB-001");
}

/// Fixture (ii): Act-and-exit (nav items → RanScript) — audit MUST be clean.
#[test]
fn matrix_act_and_exit_audit_clean() {
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_misrouted_workflow")
        .unwrap()
        .binary(menu_binary());

    let diagnostics = sim.dynamic_audit().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "fixture (ii) act-and-exit should be clean, got: {errors:#?}"
    );
}

/// Fixture (ii): Verify the route is RanScript (legitimate terminal).
#[test]
fn matrix_act_and_exit_routes_to_run_script() {
    let graph = WorkflowGraph::from_plist_file("tests/fixtures/menu_misrouted_workflow/info.plist")
        .unwrap();

    let json = r#"{"items":[{"title":"Fruits","arg":"fruits","variables":{"category":"fruits"},"autocomplete":"fruits","valid":true}]}"#;
    let screen = alfrusco::simulator::Screen::from_json(json).unwrap();
    let screen = screen.with_context(Arc::new(graph), "SF-MAIN-001".to_string());
    let action = screen.action(0).unwrap();
    // This is a legitimate act-and-exit; RanScript is correct, NOT an error.
    action.assert_runs_script();
}

/// Fixture (iii): Dangling-loopback (matched branch → nonexistent UID) — audit MUST flag ERROR.
#[test]
fn matrix_dangling_loopback_audit_flags_error() {
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_dangling_workflow")
        .unwrap()
        .binary(menu_binary());

    let diagnostics = sim.dynamic_audit().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "fixture (iii) dangling-loopback MUST flag dead-end errors"
    );
    assert!(
        errors.iter().any(|d| d.message.contains("dead-end")),
        "error message must mention dead-end, got: {errors:#?}"
    );
}

/// Fixture (iii): Verify the route is DeadEnd for the matched branch.
#[test]
fn matrix_dangling_loopback_is_dead_end() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_dangling_workflow/info.plist").unwrap();

    // arg="fruits" matches the condition, follows connection to NONEXISTENT-SF-999
    let json = r#"{"items":[{"title":"Fruits","arg":"fruits","variables":{"category":"fruits"},"autocomplete":"fruits","valid":true}]}"#;
    let screen = alfrusco::simulator::Screen::from_json(json).unwrap();
    let screen = screen.with_context(Arc::new(graph), "SF-MAIN-001".to_string());
    let action = screen.action(0).unwrap();
    assert!(
        action.is_dead_end(),
        "expected DeadEnd for dangling loopback, got: {action:?}"
    );
}
