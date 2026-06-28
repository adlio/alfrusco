//! Tests for S8: dynamic, navigation-aware audit + faithful keyword invocation.

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
fn dynamic_audit_misrouted_detects_nav_to_run_script() {
    // The misrouted fixture has SF-MAIN → RUN-SCRIPT, so nav items (which carry
    // variables + autocomplete) should be flagged as misrouted.
    let sim = Simulator::for_workflow_dir("tests/fixtures/menu_misrouted_workflow")
        .unwrap()
        .binary(menu_binary());

    let diagnostics = sim.dynamic_audit().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "expected errors for misrouted nav items, got none"
    );

    // Verify the error mentions routing to Run Script
    let has_run_script_error = errors.iter().any(|d| d.message.contains("Run Script"));
    assert!(
        has_run_script_error,
        "expected an error about routing to Run Script, got: {errors:#?}"
    );
}

#[test]
fn invoke_script_filter_uses_scriptfile() {
    // In the good menu workflow, SF-MAIN-001 has scriptfile "menu".
    // The file doesn't exist at examples/menu_workflow/menu, so it should
    // fall back to the explicit binary.
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
