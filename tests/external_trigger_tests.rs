//! Tests for External Trigger drill-in resolution (R2).
//!
//! Verifies that the routing chain:
//!   item → Conditional (loopback) → CallExternalTrigger → ExternalTrigger → Script Filter
//! correctly classifies as `DrilledIn`.

use alfrusco::simulator::{ActionResult, ObjectKind, WorkflowGraph};
use std::sync::Arc;

/// Helper: resolve action through the external trigger fixture for a given item arg.
fn resolve_with_arg(graph: &WorkflowGraph, arg: &str) -> ActionResult {
    use alfrusco::simulator::Screen;
    let json = format!(r#"{{"items":[{{"title":"Test","arg":"{arg}","valid":true}}]}}"#);
    let screen = Screen::from_json(&json).unwrap();
    let screen = screen.with_context(Arc::new(graph.clone()), "SF-MAIN-001".to_string());
    screen.action(0).unwrap()
}

fn load_graph() -> WorkflowGraph {
    WorkflowGraph::from_plist_file("tests/fixtures/menu_external_trigger_workflow/info.plist")
        .unwrap()
}

#[test]
fn external_trigger_drill_in_reports_drilled_in() {
    let graph = load_graph();
    // arg="loopback" → condition matches → CallExternalTrigger → ExternalTrigger → SF-SUB-001
    let result = resolve_with_arg(&graph, "loopback");
    assert_eq!(result.assert_drills_in(), "SF-SUB-001");
}

#[test]
fn external_trigger_else_branch_opens_url() {
    let graph = load_graph();
    // arg="anything else" → else branch → Open URL
    let result = resolve_with_arg(&graph, "https://example.com");
    assert_eq!(result.assert_opens_url(), "{query}");
}

#[test]
fn reachable_kinds_traverses_external_trigger() {
    let graph = load_graph();
    let kinds = graph.reachable_kinds("SF-MAIN-001");
    // Through the CallExternalTrigger → ExternalTrigger chain, should reach a ScriptFilter
    assert!(
        kinds.contains(&ObjectKind::ScriptFilter),
        "reachable_kinds should find ScriptFilter through external trigger chain, got: {kinds:?}"
    );
    assert!(kinds.contains(&ObjectKind::OpenUrl));
    assert!(kinds.contains(&ObjectKind::CallExternalTrigger));
    assert!(kinds.contains(&ObjectKind::ExternalTrigger));
}

#[test]
fn reaches_script_filter_via_external_trigger() {
    let graph = load_graph();
    // SF-MAIN-001 reaches SF-SUB-001 via the external trigger chain
    assert!(graph.reaches_script_filter("SF-MAIN-001"));
}

#[test]
fn parses_call_external_trigger_node() {
    let graph = load_graph();
    let node = graph.objects().get("CALL-TRIGGER-001").unwrap();
    assert_eq!(node.kind, ObjectKind::CallExternalTrigger);
    assert_eq!(
        node.config_value("externaltriggerid"),
        Some("sub-menu-trigger")
    );
}

#[test]
fn parses_external_trigger_node() {
    let graph = load_graph();
    let node = graph.objects().get("EXT-TRIGGER-001").unwrap();
    assert_eq!(node.kind, ObjectKind::ExternalTrigger);
    assert_eq!(node.config_value("triggerid"), Some("sub-menu-trigger"));
}

#[test]
fn external_trigger_uid_resolution() {
    let graph = load_graph();
    assert_eq!(
        graph.external_trigger_uid("sub-menu-trigger"),
        Some("EXT-TRIGGER-001")
    );
    assert_eq!(graph.external_trigger_uid("nonexistent"), None);
}

#[test]
fn audit_navigation_clean_for_external_trigger_workflow() {
    let graph = load_graph();
    let diagnostics = graph.audit_navigation(&[]);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= alfrusco::simulator::Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "external trigger workflow should pass audit cleanly, got: {errors:?}"
    );
}
