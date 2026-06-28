//! Tests for dynamic audit: dead-end-only error semantics (R3).
//!
//! The dynamic audit ERRORs ONLY on `DeadEnd` (dangling/unconnected matched branch).
//! Legitimate terminals (RanScript, OpenedUrl) are never flagged.

use alfrusco::simulator::{Severity, Simulator};

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

#[test]
fn dynamic_audit_good_menu_is_clean() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(menu_binary());

    let diagnostics = sim.dynamic_audit().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "expected no errors on good menu, got: {errors:#?}"
    );
}

#[test]
fn dynamic_audit_act_and_exit_not_flagged() {
    // The "misrouted" fixture connects SF-MAIN → RUN-SCRIPT directly.
    // Navigation items routing to RanScript is a legitimate act-and-exit pattern.
    // Under the faithful semantics, this is NOT an error.
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
        "act-and-exit (nav→RanScript) should NOT be flagged, got: {errors:#?}"
    );
}

#[test]
fn dynamic_audit_dangling_loopback_is_dead_end() {
    // The dangling fixture has a conditional whose loopback branch points to a
    // non-existent UID. Items with arg="loopback" (matching the condition) hit
    // a dead-end — that IS the one real error.
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
        "expected dead-end errors for dangling loopback, got none"
    );

    // Verify the error mentions dead-end
    let has_dead_end_error = errors.iter().any(|d| d.message.contains("dead-end"));
    assert!(
        has_dead_end_error,
        "expected an error about dead-end, got: {errors:#?}"
    );
}

#[test]
fn invoke_script_filter_uses_scriptfile() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(menu_binary());

    let screen = sim.invoke_script_filter("SF-MAIN-001", &[]).unwrap();
    assert!(!screen.is_empty(), "expected items from top-level menu");
    assert_eq!(screen.items()[0].title(), "Fruits");
}

#[test]
fn invoke_script_filter_sub_level() {
    let sim = Simulator::for_workflow_dir("examples/menu_workflow")
        .unwrap()
        .binary(menu_binary());

    let screen = sim.invoke_script_filter("SF-SUB-001", &["fruits"]).unwrap();
    assert!(!screen.is_empty());
    assert_eq!(screen.items()[0].title(), "Apple");
}
