use alfrusco::simulator::{ObjectKind, Severity, WorkflowGraph};

#[test]
fn menu_good_plist_parses_all_objects() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    assert_eq!(graph.objects().len(), 4);
    assert_eq!(graph.connections().len(), 3);
}

#[test]
fn menu_good_plist_finds_keyword() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    let uid = graph.script_filter_uid("menu");
    assert_eq!(uid, Some("SF-MAIN-001"));
}

#[test]
fn menu_good_plist_reaches_script_filter() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    // Main filter can reach sub-filter (via conditional)
    assert!(graph.reaches_script_filter("SF-MAIN-001"));
}

#[test]
fn menu_good_plist_reaches_open_url() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    let kinds = graph.reachable_kinds("SF-MAIN-001");
    assert!(kinds.contains(&ObjectKind::OpenUrl));
    assert!(kinds.contains(&ObjectKind::Conditional));
    assert!(kinds.contains(&ObjectKind::ScriptFilter));
}

#[test]
fn menu_good_plist_audit_has_no_errors() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    let diagnostics = graph.audit_navigation(&[]);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn menu_broken_plist_detects_dangling_connection() {
    let graph = WorkflowGraph::from_plist_file("tests/fixtures/menu_broken.plist").unwrap();
    let diagnostics = graph.audit_navigation(&[]);
    let dangling: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("non-existent"))
        .collect();
    assert!(
        !dangling.is_empty(),
        "should detect dangling connection to NONEXISTENT-UID-999"
    );
}

#[test]
fn menu_broken_plist_detects_dead_end() {
    let graph = WorkflowGraph::from_plist_file("tests/fixtures/menu_broken.plist").unwrap();
    let diagnostics = graph.audit_navigation(&[]);
    let dead_ends: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("dead-end") || d.message.contains("no outgoing"))
        .collect();
    assert!(
        !dead_ends.is_empty(),
        "should detect orphan Script Filter with no connections"
    );
}

#[test]
fn menu_broken_plist_audit_finds_main_filter_cannot_reach_action() {
    let graph = WorkflowGraph::from_plist_file("tests/fixtures/menu_broken.plist").unwrap();
    let diagnostics = graph.audit_navigation(&["menu"]);
    // The main filter connects to a non-existent UID, so reachability is empty
    let warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity >= Severity::Warning)
        .collect();
    assert!(
        !warnings.is_empty(),
        "should warn that 'menu' filter cannot reach anything useful"
    );
}

#[test]
fn outgoing_connections_with_modifier_filter() {
    let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
    // Default connections (modifier 0)
    let conns = graph.outgoing_connections("SF-MAIN-001", Some(0));
    assert_eq!(conns.len(), 1);
    // Non-existent modifier
    let conns = graph.outgoing_connections("SF-MAIN-001", Some(1_048_576));
    assert!(conns.is_empty());
    // All connections (no filter)
    let conns = graph.outgoing_connections("SF-MAIN-001", None);
    assert_eq!(conns.len(), 1);
}
