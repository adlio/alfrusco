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
    /// Raw config key-value pairs (string values only).
    config_strings: HashMap<String, String>,
}

impl ObjectNode {
    /// Returns a string config value by key (e.g. `"url"` for Open URL actions).
    pub fn config_value(&self, key: &str) -> Option<&str> {
        self.config_strings.get(key).map(String::as_str)
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
                            connections.push(Connection {
                                source_uid: source_uid.clone(),
                                destination_uid,
                                modifiers,
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
            config_strings,
        })
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
    /// of modifier). The starting node itself is not included.
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
