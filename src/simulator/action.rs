//! Action routing: what happens when a user actions an item.
//!
//! Given a rendered [`Screen`](super::Screen) (from a specific Script Filter) and the
//! workflow graph, determines the outcome of actioning an item — drill-in to another
//! Script Filter, open a URL, run a script, autocomplete loopback, or dead-end.

use super::graph::{ObjectKind, WorkflowGraph};

/// The outcome of actioning an item from a Script Filter screen.
///
/// Determined by walking the workflow graph from the source Script Filter,
/// following connections (optionally gated by modifier keys) through conditionals
/// to the final destination.
///
/// # Example
///
/// ```no_run
/// use alfrusco::simulator::ActionResult;
///
/// # let result = ActionResult::DeadEnd;
/// match &result {
///     ActionResult::DrilledIn { target_uid } => {
///         println!("Would drill into Script Filter: {target_uid}");
///     }
///     ActionResult::OpenedUrl { url_template } => {
///         println!("Would open URL: {url_template}");
///     }
///     ActionResult::RanScript { target_uid } => {
///         println!("Would run script at: {target_uid}");
///     }
///     ActionResult::TypedAutocomplete { text } => {
///         println!("Would autocomplete to: {text}");
///     }
///     ActionResult::DeadEnd => {
///         println!("No route found (dead-end)");
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    /// The item routes to another Script Filter (drill-in / loopback navigation).
    DrilledIn {
        /// The UID of the destination Script Filter.
        target_uid: String,
    },
    /// The item routes to an Open URL action.
    OpenedUrl {
        /// The URL template from the action's config (e.g. `{query}`).
        url_template: String,
    },
    /// The item routes to a Run Script action.
    RanScript {
        /// The UID of the Run Script object.
        target_uid: String,
    },
    /// The item is not valid and has an autocomplete string (loopback to same filter).
    TypedAutocomplete {
        /// The autocomplete text that would be fed back to the Script Filter.
        text: String,
    },
    /// No route found from the source Script Filter (broken graph or no connection).
    DeadEnd,
}

impl ActionResult {
    /// Asserts that this action drills in to another Script Filter.
    ///
    /// # Panics
    ///
    /// Panics if this is not a `DrilledIn` result.
    pub fn assert_drills_in(&self) -> &str {
        match self {
            Self::DrilledIn { target_uid } => target_uid,
            other => panic!("expected DrilledIn, got {other:?}"),
        }
    }

    /// Asserts that this action opens a URL.
    ///
    /// # Panics
    ///
    /// Panics if this is not an `OpenedUrl` result.
    pub fn assert_opens_url(&self) -> &str {
        match self {
            Self::OpenedUrl { url_template } => url_template,
            other => panic!("expected OpenedUrl, got {other:?}"),
        }
    }

    /// Asserts that this action runs a script.
    ///
    /// # Panics
    ///
    /// Panics if this is not a `RanScript` result.
    pub fn assert_runs_script(&self) -> &str {
        match self {
            Self::RanScript { target_uid } => target_uid,
            other => panic!("expected RanScript, got {other:?}"),
        }
    }

    /// Asserts that this action is an autocomplete loopback.
    ///
    /// # Panics
    ///
    /// Panics if this is not a `TypedAutocomplete` result.
    pub fn assert_autocompletes(&self) -> &str {
        match self {
            Self::TypedAutocomplete { text } => text,
            other => panic!("expected TypedAutocomplete, got {other:?}"),
        }
    }

    /// Returns `true` if this action results in a dead-end.
    pub fn is_dead_end(&self) -> bool {
        matches!(self, Self::DeadEnd)
    }
}

/// Walks the graph from a source Script Filter to determine the action outcome.
///
/// The routing logic:
/// 1. If the item has `valid: false` and an `autocomplete` string, it's a loopback.
/// 2. Otherwise, follow outgoing connections from the source (filtered by modifier).
/// 3. Walk through Conditional nodes transparently (they pass through to their outputs).
/// 4. The first non-conditional destination determines the result type.
pub(crate) fn resolve_action(
    graph: &WorkflowGraph,
    source_uid: &str,
    item_valid: bool,
    item_autocomplete: Option<&str>,
    modifiers: u64,
) -> ActionResult {
    // Autocomplete loopback: valid:false + autocomplete set
    if !item_valid {
        if let Some(text) = item_autocomplete {
            return ActionResult::TypedAutocomplete {
                text: text.to_string(),
            };
        }
    }

    // Walk graph from source following modifier-matched connections
    resolve_from_node(graph, source_uid, modifiers, &mut Vec::new())
}

/// Recursively resolves the action by walking through conditionals.
fn resolve_from_node(
    graph: &WorkflowGraph,
    uid: &str,
    modifiers: u64,
    visited: &mut Vec<String>,
) -> ActionResult {
    // Prevent cycles
    if visited.contains(&uid.to_string()) {
        return ActionResult::DeadEnd;
    }
    visited.push(uid.to_string());

    // Get outgoing connections matching the modifier
    let connections = graph.outgoing_connections(uid, Some(modifiers));

    // If no modifier-specific connection, try default (0) as fallback
    let connections = if connections.is_empty() && modifiers != 0 {
        graph.outgoing_connections(uid, Some(0))
    } else {
        connections
    };

    if connections.is_empty() {
        return ActionResult::DeadEnd;
    }

    // Follow the first matching connection
    let dest_uid = &connections[0].destination_uid;

    // Look up the destination node
    let Some(dest_node) = graph.objects().get(dest_uid) else {
        // Dangling connection → dead-end
        return ActionResult::DeadEnd;
    };

    match &dest_node.kind {
        ObjectKind::ScriptFilter => ActionResult::DrilledIn {
            target_uid: dest_uid.clone(),
        },
        ObjectKind::OpenUrl => {
            let url_template = dest_node
                .config_value("url")
                .unwrap_or_default()
                .to_string();
            ActionResult::OpenedUrl { url_template }
        }
        ObjectKind::RunScript => ActionResult::RanScript {
            target_uid: dest_uid.clone(),
        },
        ObjectKind::Conditional => {
            // Walk through the conditional transparently
            resolve_from_node(graph, dest_uid, modifiers, visited)
        }
        // Clipboard, other — treat as RanScript (it's an action endpoint)
        ObjectKind::Clipboard | ObjectKind::Other(_) | ObjectKind::Keyword => {
            ActionResult::RanScript {
                target_uid: dest_uid.clone(),
            }
        }
    }
}
