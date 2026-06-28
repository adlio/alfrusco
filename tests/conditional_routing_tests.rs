//! Tests for matchstring-faithful conditional routing (R1).

use alfrusco::simulator::{ActionResult, MatchMode, WorkflowGraph};
use std::sync::Arc;

/// Helper: resolve action through the conditional fixture for a given item arg.
fn resolve_with_arg(graph: &WorkflowGraph, arg: &str) -> ActionResult {
    use alfrusco::simulator::Screen;
    let json = format!(r#"{{"items":[{{"title":"Test","arg":"{arg}","valid":true}}]}}"#);
    let screen = Screen::from_json(&json).unwrap();
    let screen = screen.with_context(Arc::new(graph.clone()), "SF-MAIN-001".to_string());
    screen.action(0).unwrap()
}

#[test]
fn conditional_routes_loopback_arg_to_script_filter() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    let result = resolve_with_arg(&graph, "loopback");
    assert_eq!(result.assert_drills_in(), "SF-SUB-001");
}

#[test]
fn conditional_routes_url_arg_to_open_url() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    let result = resolve_with_arg(&graph, "https://example.com");
    assert_eq!(result.assert_opens_url(), "{query}");
}

#[test]
fn conditional_routes_unmatched_arg_to_else_branch() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    let result = resolve_with_arg(&graph, "run");
    result.assert_runs_script();
}

#[test]
fn conditional_case_insensitive_match() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    // "LOOPBACK" should match condition[0] (matchmode=0/Is, case-insensitive)
    let result = resolve_with_arg(&graph, "LOOPBACK");
    assert_eq!(result.assert_drills_in(), "SF-SUB-001");
}

#[test]
fn conditional_starts_with_partial_url() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    // "http://example.com" starts with "http" → routes to URL action
    let result = resolve_with_arg(&graph, "http://example.com");
    assert_eq!(result.assert_opens_url(), "{query}");
}

// --- MatchMode unit tests ---

#[test]
fn matchmode_is_exact() {
    assert!(MatchMode::Is.evaluate("hello", "hello", false));
    assert!(MatchMode::Is.evaluate("Hello", "hello", false));
    assert!(!MatchMode::Is.evaluate("Hello", "hello", true));
    assert!(!MatchMode::Is.evaluate("hello", "world", false));
}

#[test]
fn matchmode_is_empty_check() {
    // Empty matchstring → "is empty" semantics
    assert!(MatchMode::Is.evaluate("", "", false));
    assert!(!MatchMode::Is.evaluate("notempty", "", false));
}

#[test]
fn matchmode_is_not() {
    assert!(MatchMode::IsNot.evaluate("hello", "world", false));
    assert!(!MatchMode::IsNot.evaluate("hello", "hello", false));
    // Empty matchstring → "is not empty" semantics
    assert!(MatchMode::IsNot.evaluate("notempty", "", false));
    assert!(!MatchMode::IsNot.evaluate("", "", false));
}

#[test]
fn matchmode_contains() {
    assert!(MatchMode::Contains.evaluate("hello world", "world", false));
    assert!(MatchMode::Contains.evaluate("Hello World", "world", false));
    assert!(!MatchMode::Contains.evaluate("Hello World", "world", true));
    assert!(!MatchMode::Contains.evaluate("hello", "xyz", false));
}

#[test]
fn matchmode_does_not_contain() {
    assert!(MatchMode::DoesNotContain.evaluate("hello", "xyz", false));
    assert!(!MatchMode::DoesNotContain.evaluate("hello world", "world", false));
}

#[test]
fn matchmode_starts_with() {
    assert!(MatchMode::StartsWith.evaluate("http://x.com", "http", false));
    assert!(MatchMode::StartsWith.evaluate("HTTP://x.com", "http", false));
    assert!(!MatchMode::StartsWith.evaluate("HTTP://x.com", "http", true));
    assert!(!MatchMode::StartsWith.evaluate("ftp://x.com", "http", false));
}

#[test]
fn matchmode_ends_with() {
    assert!(MatchMode::EndsWith.evaluate("file.txt", ".txt", false));
    assert!(MatchMode::EndsWith.evaluate("file.TXT", ".txt", false));
    assert!(!MatchMode::EndsWith.evaluate("file.TXT", ".txt", true));
    assert!(!MatchMode::EndsWith.evaluate("file.rs", ".txt", false));
}

#[test]
fn matchmode_regex() {
    assert!(MatchMode::MatchesRegex.evaluate("abc123", r"\d+", false));
    assert!(!MatchMode::MatchesRegex.evaluate("abc", r"\d+", false));
    assert!(MatchMode::MatchesRegex.evaluate("Hello", "hello", false)); // case-insensitive
    assert!(!MatchMode::MatchesRegex.evaluate("Hello", "^hello$", true)); // case-sensitive
}

#[test]
fn matchmode_from_integer_roundtrip() {
    assert_eq!(MatchMode::from_integer(0), MatchMode::Is);
    assert_eq!(MatchMode::from_integer(1), MatchMode::IsNot);
    assert_eq!(MatchMode::from_integer(2), MatchMode::Contains);
    assert_eq!(MatchMode::from_integer(3), MatchMode::DoesNotContain);
    assert_eq!(MatchMode::from_integer(4), MatchMode::StartsWith);
    assert_eq!(MatchMode::from_integer(5), MatchMode::EndsWith);
    assert_eq!(MatchMode::from_integer(6), MatchMode::MatchesRegex);
    assert_eq!(MatchMode::from_integer(99), MatchMode::Unknown(99));
}

// --- Graph parsing tests ---

#[test]
fn parses_conditions_from_conditional_node() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    let cond_node = graph.objects().get("COND-001").unwrap();
    assert_eq!(cond_node.conditions.len(), 2);
    assert_eq!(
        cond_node.conditions[0].uid.as_deref(),
        Some("COND-OUT-LOOPBACK")
    );
    assert_eq!(cond_node.conditions[0].match_string, "loopback");
    assert_eq!(cond_node.conditions[0].match_mode, MatchMode::Is);
    assert_eq!(cond_node.conditions[1].uid.as_deref(), Some("COND-OUT-URL"));
    assert_eq!(cond_node.conditions[1].match_string, "http");
    assert_eq!(cond_node.conditions[1].match_mode, MatchMode::StartsWith);
}

#[test]
fn parses_source_output_uid_from_connections() {
    let graph =
        WorkflowGraph::from_plist_file("tests/fixtures/menu_conditional_workflow/info.plist")
            .unwrap();
    let cond_conns = graph.outgoing_connections("COND-001", None);
    assert_eq!(cond_conns.len(), 3);
    // At least one should have COND-OUT-LOOPBACK
    let loopback_conn = cond_conns
        .iter()
        .find(|c| c.source_output_uid.as_deref() == Some("COND-OUT-LOOPBACK"))
        .unwrap();
    assert_eq!(loopback_conn.destination_uid, "SF-SUB-001");
}
