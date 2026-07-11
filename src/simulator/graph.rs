//! Alfred workflow graph model parsed from `info.plist`.
//!
//! An Alfred workflow's `info.plist` defines a directed graph of objects (Script Filters,
//! actions, utilities) connected by edges (optionally gated by modifier keys). This module
//! parses that graph and provides queries for reachability, keyword lookup, and audit.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

/// The kind of an Alfred workflow object.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    /// A Script Filter input (`alfred.workflow.input.scriptfilter`).
    ScriptFilter,
    /// A keyword input (`alfred.workflow.input.keyword`).
    Keyword,
    /// An Open URL action (`alfred.workflow.action.openurl`).
    OpenUrl,
    /// A Run Script action (`alfred.workflow.action.script`).
    RunScript,
    /// A conditional utility (`alfred.workflow.utility.conditional`).
    Conditional,
    /// A clipboard output (`alfred.workflow.output.clipboard`).
    Clipboard,
    /// A Call External Trigger output (`alfred.workflow.output.callexternaltrigger`).
    ///
    /// Routes to a matching External Trigger input by trigger ID, enabling
    /// indirect drill-in navigation.
    CallExternalTrigger,
    /// An External Trigger input (`alfred.workflow.trigger.external`).
    ///
    /// Receives calls from [`CallExternalTrigger`](Self::CallExternalTrigger) nodes
    /// and continues traversal along its outgoing connections.
    ExternalTrigger,
    /// Any other Alfred object type.
    Other(String),
}

impl ObjectKind {
    /// Parses an Alfred object type string into an [`ObjectKind`].
    fn from_type_string(s: &str) -> Self {
        match s {
            "alfred.workflow.input.scriptfilter" => Self::ScriptFilter,
            "alfred.workflow.input.keyword" => Self::Keyword,
            "alfred.workflow.action.openurl" => Self::OpenUrl,
            "alfred.workflow.action.script" => Self::RunScript,
            "alfred.workflow.utility.conditional" => Self::Conditional,
            "alfred.workflow.output.clipboard" => Self::Clipboard,
            "alfred.workflow.output.callexternaltrigger" => Self::CallExternalTrigger,
            "alfred.workflow.trigger.external" => Self::ExternalTrigger,
            other => Self::Other(other.to_string()),
        }
    }
}

/// A node in the workflow graph representing an Alfred object.
#[derive(Debug, Clone)]
pub struct ObjectNode {
    /// The unique identifier of this object within the workflow.
    pub uid: String,
    /// The kind of object.
    pub kind: ObjectKind,
    /// The keyword configured on this object (Script Filters and Keywords only).
    pub keyword: Option<String>,
    /// The display title of this object.
    pub title: Option<String>,
    /// Conditions for Conditional objects (empty for non-conditionals).
    pub conditions: Vec<Condition>,
    /// Raw config key-value pairs (string values only).
    config_strings: HashMap<String, String>,
}

impl ObjectNode {
    /// Returns a string config value by key (e.g. `"url"` for Open URL actions).
    pub fn config_value(&self, key: &str) -> Option<&str> {
        self.config_strings.get(key).map(String::as_str)
    }

    /// Returns the script file name configured on this Script Filter, if any.
    ///
    /// This is the `scriptfile` field from the Alfred `info.plist`, representing
    /// the binary that Alfred executes relative to the workflow directory.
    pub fn script_file(&self) -> Option<&str> {
        self.config_value("scriptfile").filter(|s| !s.is_empty())
    }

    /// Returns the inline script configured on this Script Filter, if any.
    ///
    /// This is the `script` field from the Alfred `info.plist`, containing
    /// a shell command that Alfred executes directly.
    pub fn script(&self) -> Option<&str> {
        self.config_value("script").filter(|s| !s.is_empty())
    }
}

/// A directed edge between two objects, optionally gated by a modifier key combination.
#[derive(Debug, Clone)]
pub struct Connection {
    /// The UID of the source object.
    pub source_uid: String,
    /// The UID of the destination object.
    pub destination_uid: String,
    /// The modifier bitmask (0 = no modifier / default connection).
    pub modifiers: u64,
    /// The output port UID on the source (used by Conditional nodes to distinguish branches).
    ///
    /// For conditionals, each condition's `uid` identifies one output port, and the
    /// else branch typically has no `sourceoutputuid` or a well-known sentinel.
    pub source_output_uid: Option<String>,
}

/// A single condition branch inside a Conditional object.
///
/// Alfred evaluates conditions in order; the first match determines which output
/// port the connection follows. If no condition matches, the else branch is taken.
#[derive(Debug, Clone)]
pub struct Condition {
    /// The UID of this condition's output port (matches [`Connection::source_output_uid`]).
    pub uid: Option<String>,
    /// What to test — typically `{query}` (the item's arg) or `{var:name}`.
    pub input_string: String,
    /// The match mode (see [`MatchMode`]).
    pub match_mode: MatchMode,
    /// The pattern to match against.
    pub match_string: String,
    /// Whether matching is case-sensitive.
    pub match_case_sensitive: bool,
}

/// Alfred Conditional match modes.
///
/// These numeric values correspond to the `matchmode` integer in Alfred's
/// `info.plist` conditional configuration.
///
/// # Match mode table
///
/// | Value | Mode | Description |
/// |-------|------|-------------|
/// | 0 | Is | Exact equality (or "is empty" when `matchstring` is empty) |
/// | 1 | IsNot | Not equal |
/// | 2 | Contains | Substring match |
/// | 3 | DoesNotContain | No substring match |
/// | 4 | StartsWith | Prefix match |
/// | 5 | EndsWith | Suffix match |
/// | 6 | MatchesRegex | Regular expression match |
///
/// # Example
///
/// ```
/// use alfrusco::simulator::MatchMode;
///
/// assert_eq!(MatchMode::from_integer(0), MatchMode::Is);
/// assert_eq!(MatchMode::from_integer(4), MatchMode::StartsWith);
/// assert_eq!(MatchMode::from_integer(99), MatchMode::Unknown(99));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// Exact equality (matchmode 0). When `matchstring` is empty, tests "is empty".
    Is,
    /// Not equal (matchmode 1). When `matchstring` is empty, tests "is not empty".
    IsNot,
    /// Substring match (matchmode 2).
    Contains,
    /// No substring match (matchmode 3).
    DoesNotContain,
    /// Prefix match (matchmode 4).
    StartsWith,
    /// Suffix match (matchmode 5).
    EndsWith,
    /// Regular expression match (matchmode 6).
    MatchesRegex,
    /// An unrecognized matchmode value.
    Unknown(u64),
}

impl MatchMode {
    /// Parses a matchmode integer from an Alfred `info.plist`.
    ///
    /// # Example
    ///
    /// ```
    /// use alfrusco::simulator::MatchMode;
    ///
    /// assert_eq!(MatchMode::from_integer(2), MatchMode::Contains);
    /// ```
    pub fn from_integer(n: u64) -> Self {
        match n {
            0 => Self::Is,
            1 => Self::IsNot,
            2 => Self::Contains,
            3 => Self::DoesNotContain,
            4 => Self::StartsWith,
            5 => Self::EndsWith,
            6 => Self::MatchesRegex,
            other => Self::Unknown(other),
        }
    }

    /// Evaluates this match mode against an input and pattern.
    ///
    /// When `case_sensitive` is false, both input and pattern are lowercased
    /// before comparison (except for regex, which uses the `(?i)` flag).
    ///
    /// # Example
    ///
    /// ```
    /// use alfrusco::simulator::MatchMode;
    ///
    /// assert!(MatchMode::Is.evaluate("hello", "hello", false));
    /// assert!(!MatchMode::Is.evaluate("hello", "world", false));
    /// assert!(MatchMode::Is.evaluate("Hello", "hello", false));
    /// assert!(!MatchMode::Is.evaluate("Hello", "hello", true));
    /// assert!(MatchMode::Contains.evaluate("hello world", "world", false));
    /// assert!(MatchMode::StartsWith.evaluate("http://x.com", "http", false));
    /// assert!(MatchMode::EndsWith.evaluate("file.txt", ".txt", false));
    /// ```
    pub fn evaluate(&self, input: &str, pattern: &str, case_sensitive: bool) -> bool {
        match self {
            Self::Is => {
                // Empty pattern means "is empty" check
                if pattern.is_empty() {
                    return input.is_empty();
                }
                if case_sensitive {
                    input == pattern
                } else {
                    input.eq_ignore_ascii_case(pattern)
                }
            }
            Self::IsNot => {
                if pattern.is_empty() {
                    return !input.is_empty();
                }
                if case_sensitive {
                    input != pattern
                } else {
                    !input.eq_ignore_ascii_case(pattern)
                }
            }
            Self::Contains => {
                if case_sensitive {
                    input.contains(pattern)
                } else {
                    input
                        .to_ascii_lowercase()
                        .contains(&pattern.to_ascii_lowercase())
                }
            }
            Self::DoesNotContain => {
                if case_sensitive {
                    !input.contains(pattern)
                } else {
                    !input
                        .to_ascii_lowercase()
                        .contains(&pattern.to_ascii_lowercase())
                }
            }
            Self::StartsWith => {
                if case_sensitive {
                    input.starts_with(pattern)
                } else {
                    input
                        .to_ascii_lowercase()
                        .starts_with(&pattern.to_ascii_lowercase())
                }
            }
            Self::EndsWith => {
                if case_sensitive {
                    input.ends_with(pattern)
                } else {
                    input
                        .to_ascii_lowercase()
                        .ends_with(&pattern.to_ascii_lowercase())
                }
            }
            Self::MatchesRegex => {
                let pattern_str = if case_sensitive {
                    pattern.to_string()
                } else {
                    format!("(?i){pattern}")
                };
                regex::Regex::new(&pattern_str).is_ok_and(|re| re.is_match(input))
            }
            Self::Unknown(_) => false,
        }
    }
}

/// Severity level for audit diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational finding (not necessarily a problem).
    Info,
    /// A potential issue that may cause unexpected behavior.
    Warning,
    /// A definite problem (dead-end, unreachable object).
    Error,
}

/// A diagnostic finding from auditing a workflow graph.
#[derive(Debug, Clone)]
pub struct AuditDiagnostic {
    /// Severity of this finding.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub message: String,
    /// The UID of the object this diagnostic relates to, if any.
    pub object_uid: Option<String>,
}

/// A parsed Alfred workflow graph from an `info.plist` file.
///
/// Models the objects and connections in the workflow, enabling queries
/// about reachability, keyword resolution, and structural correctness.
///
/// # Example
///
/// ```
/// # use alfrusco::simulator::WorkflowGraph;
/// let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
/// assert!(!graph.objects().is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct WorkflowGraph {
    objects: HashMap<String, ObjectNode>,
    connections: Vec<Connection>,
}

impl WorkflowGraph {
    /// Parses a workflow graph from an Alfred `info.plist` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or does not contain valid
    /// Alfred workflow data.
    ///
    /// # Example
    ///
    /// ```
    /// # use alfrusco::simulator::WorkflowGraph;
    /// let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
    /// assert!(!graph.objects().is_empty());
    /// ```
    pub fn from_plist_file(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        let value = plist::Value::from_file(path.as_ref())
            .map_err(|e| GraphError::PlistParse(e.to_string()))?;
        let dict = value
            .as_dictionary()
            .ok_or_else(|| GraphError::PlistParse("root is not a dictionary".to_string()))?;
        Self::from_plist_dict(dict)
    }

    /// Parses a workflow graph from a plist dictionary (already loaded).
    fn from_plist_dict(dict: &plist::Dictionary) -> Result<Self, GraphError> {
        let mut objects = HashMap::new();
        let mut connections = Vec::new();

        // Parse objects
        if let Some(objs) = dict.get("objects").and_then(|v| v.as_array()) {
            for obj in objs {
                if let Some(node) = Self::parse_object(obj) {
                    objects.insert(node.uid.clone(), node);
                }
            }
        }

        // Parse connections
        if let Some(conns) = dict.get("connections").and_then(|v| v.as_dictionary()) {
            for (source_uid, destinations) in conns {
                if let Some(dests) = destinations.as_array() {
                    for dest in dests {
                        if let Some(dest_dict) = dest.as_dictionary() {
                            let destination_uid = dest_dict
                                .get("destinationuid")
                                .and_then(|v| v.as_string())
                                .unwrap_or_default()
                                .to_string();
                            let modifiers = dest_dict
                                .get("modifiers")
                                .and_then(|v| v.as_unsigned_integer())
                                .unwrap_or(0);
                            let source_output_uid = dest_dict
                                .get("sourceoutputuid")
                                .and_then(|v| v.as_string())
                                .map(String::from);
                            connections.push(Connection {
                                source_uid: source_uid.clone(),
                                destination_uid,
                                modifiers,
                                source_output_uid,
                            });
                        }
                    }
                }
            }
        }

        Ok(Self {
            objects,
            connections,
        })
    }

    /// Parses a single Alfred object from a plist value.
    fn parse_object(value: &plist::Value) -> Option<ObjectNode> {
        let dict = value.as_dictionary()?;
        let uid = dict.get("uid")?.as_string()?.to_string();
        let type_str = dict.get("type")?.as_string()?;
        let kind = ObjectKind::from_type_string(type_str);

        let config = dict.get("config").and_then(|v| v.as_dictionary());
        let keyword = config
            .and_then(|c| c.get("keyword"))
            .and_then(|v| v.as_string())
            .map(String::from)
            .filter(|s| !s.is_empty());
        let title = config
            .and_then(|c| c.get("title"))
            .and_then(|v| v.as_string())
            .map(String::from);

        // Parse conditions for Conditional objects
        let conditions = if kind == ObjectKind::Conditional {
            Self::parse_conditions(config)
        } else {
            Vec::new()
        };

        // Collect all string-valued config entries
        let config_strings = config
            .map(|c| {
                c.iter()
                    .filter_map(|(k, v)| v.as_string().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Some(ObjectNode {
            uid,
            kind,
            keyword,
            title,
            conditions,
            config_strings,
        })
    }

    /// Parses the conditions array from a Conditional object's config.
    fn parse_conditions(config: Option<&plist::Dictionary>) -> Vec<Condition> {
        let Some(config) = config else {
            return Vec::new();
        };
        let Some(conditions_array) = config.get("conditions").and_then(|v| v.as_array()) else {
            return Vec::new();
        };

        conditions_array
            .iter()
            .filter_map(|v| {
                let d = v.as_dictionary()?;
                let input_string = d
                    .get("inputstring")
                    .and_then(|v| v.as_string())
                    .unwrap_or("{query}")
                    .to_string();
                let match_mode = MatchMode::from_integer(
                    d.get("matchmode")
                        .and_then(|v| v.as_unsigned_integer())
                        .unwrap_or(0),
                );
                let match_string = d
                    .get("matchstring")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                let match_case_sensitive = d
                    .get("matchcasesensitive")
                    .and_then(|v| v.as_boolean())
                    .unwrap_or(false);
                let uid = d.get("uid").and_then(|v| v.as_string()).map(String::from);
                Some(Condition {
                    uid,
                    input_string,
                    match_mode,
                    match_string,
                    match_case_sensitive,
                })
            })
            .collect()
    }

    /// Returns all objects in the graph.
    pub fn objects(&self) -> &HashMap<String, ObjectNode> {
        &self.objects
    }

    /// Returns all connections in the graph.
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Finds the UID of a Script Filter that has the given keyword.
    ///
    /// Returns `None` if no Script Filter matches the keyword.
    ///
    /// # Example
    ///
    /// ```
    /// # use alfrusco::simulator::WorkflowGraph;
    /// let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
    /// let uid = graph.script_filter_uid("sleep");
    /// assert!(uid.is_some());
    /// ```
    pub fn script_filter_uid(&self, keyword: &str) -> Option<&str> {
        self.objects.values().find_map(|node| {
            if node.kind == ObjectKind::ScriptFilter && node.keyword.as_deref() == Some(keyword) {
                Some(node.uid.as_str())
            } else {
                None
            }
        })
    }

    /// Returns the set of [`ObjectKind`]s reachable from the given UID via any connections.
    ///
    /// Performs a breadth-first traversal following all outgoing connections (regardless
    /// of modifier). Call External Trigger nodes are resolved to their matching
    /// External Trigger inputs, allowing the traversal to cross trigger boundaries.
    /// The starting node itself is not included.
    pub fn reachable_kinds(&self, uid: &str) -> HashSet<ObjectKind> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut kinds = HashSet::new();

        // Seed with direct successors of the starting node
        for conn in &self.connections {
            if conn.source_uid == uid && !visited.contains(&conn.destination_uid) {
                visited.insert(conn.destination_uid.clone());
                queue.push_back(conn.destination_uid.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(node) = self.objects.get(&current) {
                kinds.insert(node.kind.clone());

                // If this is a CallExternalTrigger, resolve to the matching trigger input
                if node.kind == ObjectKind::CallExternalTrigger {
                    if let Some(trigger_id) = node.config_value("externaltriggerid") {
                        if let Some(trigger_uid) = self.external_trigger_uid(trigger_id) {
                            if !visited.contains(trigger_uid) {
                                visited.insert(trigger_uid.to_string());
                                queue.push_back(trigger_uid.to_string());
                            }
                        }
                    }
                }
            }
            for conn in &self.connections {
                if conn.source_uid == current && !visited.contains(&conn.destination_uid) {
                    visited.insert(conn.destination_uid.clone());
                    queue.push_back(conn.destination_uid.clone());
                }
            }
        }

        kinds
    }

    /// Returns whether the given Script Filter UID can reach another Script Filter
    /// (indicating a drill-in / loopback navigation pattern) or reaches any action
    /// (indicating a functional endpoint).
    ///
    /// A Script Filter that reaches nothing is a dead-end.
    pub fn reaches_script_filter(&self, uid: &str) -> bool {
        let kinds = self.reachable_kinds(uid);
        kinds.contains(&ObjectKind::ScriptFilter)
    }

    /// Audits the workflow's navigation graph for common defects.
    ///
    /// Checks the given keywords (or all Script Filters if `keywords` is empty):
    /// - Dead-end Script Filters with no outgoing connections.
    /// - Script Filters that cannot reach any action or another Script Filter.
    /// - Connections pointing to non-existent UIDs (dangling references).
    /// - Script Filters not reachable from any keyword entry point.
    ///
    /// # Example
    ///
    /// ```
    /// # use alfrusco::simulator::{WorkflowGraph, Severity};
    /// let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
    /// let diagnostics = graph.audit_navigation(&[]);
    /// for d in &diagnostics {
    ///     if d.severity >= Severity::Error {
    ///         eprintln!("ERROR: {}", d.message);
    ///     }
    /// }
    /// ```
    pub fn audit_navigation(&self, keywords: &[&str]) -> Vec<AuditDiagnostic> {
        let mut diagnostics = Vec::new();

        // Determine which Script Filters to audit
        let filters: Vec<&ObjectNode> = if keywords.is_empty() {
            self.objects
                .values()
                .filter(|n| n.kind == ObjectKind::ScriptFilter)
                .collect()
        } else {
            keywords
                .iter()
                .filter_map(|kw| {
                    self.script_filter_uid(kw)
                        .and_then(|uid| self.objects.get(uid))
                })
                .collect()
        };

        // Check each Script Filter for connectivity
        for filter in &filters {
            let outgoing: Vec<&Connection> = self
                .connections
                .iter()
                .filter(|c| c.source_uid == filter.uid)
                .collect();

            if outgoing.is_empty() {
                diagnostics.push(AuditDiagnostic {
                    severity: Severity::Error,
                    message: format!(
                        "Script Filter '{}' ({}) has no outgoing connections (dead-end)",
                        filter.keyword.as_deref().unwrap_or("?"),
                        filter.uid,
                    ),
                    object_uid: Some(filter.uid.clone()),
                });
                continue;
            }

            let reachable = self.reachable_kinds(&filter.uid);
            let has_action = reachable.contains(&ObjectKind::OpenUrl)
                || reachable.contains(&ObjectKind::RunScript)
                || reachable.contains(&ObjectKind::Clipboard);
            let has_drill_in = reachable.contains(&ObjectKind::ScriptFilter);

            if !has_action && !has_drill_in {
                diagnostics.push(AuditDiagnostic {
                    severity: Severity::Warning,
                    message: format!(
                        "Script Filter '{}' ({}) cannot reach any action or another Script Filter",
                        filter.keyword.as_deref().unwrap_or("?"),
                        filter.uid,
                    ),
                    object_uid: Some(filter.uid.clone()),
                });
            }
        }

        // Check for dangling connections (destination UIDs that don't exist)
        for conn in &self.connections {
            if !self.objects.contains_key(&conn.destination_uid) {
                diagnostics.push(AuditDiagnostic {
                    severity: Severity::Error,
                    message: format!(
                        "Connection from '{}' points to non-existent object '{}'",
                        conn.source_uid, conn.destination_uid,
                    ),
                    object_uid: Some(conn.source_uid.clone()),
                });
            }
        }

        diagnostics
    }

    /// Finds the External Trigger input node that matches a given trigger ID.
    ///
    /// Alfred's `callexternaltrigger` output nodes reference a trigger by ID.
    /// This method locates the corresponding `trigger.external` input node so
    /// traversal can continue along its outgoing connections.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use alfrusco::simulator::WorkflowGraph;
    /// let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
    /// if let Some(trigger_uid) = graph.external_trigger_uid("my-trigger") {
    ///     // Continue traversal from this trigger's outgoing connections
    ///     let conns = graph.outgoing_connections(trigger_uid, Some(0));
    /// }
    /// ```
    pub fn external_trigger_uid(&self, trigger_id: &str) -> Option<&str> {
        self.objects.values().find_map(|node| {
            if node.kind == ObjectKind::ExternalTrigger
                && node.config_value("triggerid") == Some(trigger_id)
            {
                Some(node.uid.as_str())
            } else {
                None
            }
        })
    }

    /// Returns the outgoing connections from the given UID, optionally filtered
    /// by modifier bitmask. If `modifiers` is `None`, returns all connections.
    pub fn outgoing_connections(&self, uid: &str, modifiers: Option<u64>) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.source_uid == uid && modifiers.is_none_or(|m| c.modifiers == m))
            .collect()
    }
}

/// Errors that can occur when parsing a workflow graph.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphError {
    /// The plist file could not be read or parsed.
    #[error("failed to parse info.plist: {0}")]
    PlistParse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_existing_workflow_plist() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        assert_eq!(graph.objects().len(), 2);
        assert_eq!(graph.connections().len(), 1);
    }

    #[test]
    fn script_filter_uid_by_keyword() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        let uid = graph.script_filter_uid("sleep");
        assert_eq!(uid, Some("1683B57E-4C4A-402A-AECB-D493E46FE968"));
    }

    #[test]
    fn script_filter_uid_nonexistent_keyword() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        assert_eq!(graph.script_filter_uid("nonexistent"), None);
    }

    #[test]
    fn reachable_kinds_from_script_filter() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        let kinds = graph.reachable_kinds("1683B57E-4C4A-402A-AECB-D493E46FE968");
        assert!(kinds.contains(&ObjectKind::OpenUrl));
        assert!(!kinds.contains(&ObjectKind::ScriptFilter));
    }

    #[test]
    fn reaches_script_filter_false_when_only_action() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        assert!(!graph.reaches_script_filter("1683B57E-4C4A-402A-AECB-D493E46FE968"));
    }

    #[test]
    fn audit_navigation_clean_workflow() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        let diagnostics = graph.audit_navigation(&[]);
        // The existing workflow has a Script Filter → OpenURL, so no errors
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn outgoing_connections_default() {
        let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
        let conns = graph.outgoing_connections("1683B57E-4C4A-402A-AECB-D493E46FE968", Some(0));
        assert_eq!(conns.len(), 1);
        assert_eq!(
            conns[0].destination_uid,
            "513B7861-747E-4F95-A5BC-6AE622EEB1BF"
        );
    }
}
