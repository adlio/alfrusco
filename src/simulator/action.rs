//! Action routing: what happens when a user actions an item.
//!
//! Given a rendered [`Screen`](super::Screen) (from a specific Script Filter) and the
//! workflow graph, determines the outcome of actioning an item — drill-in to another
//! Script Filter, open a URL, run a script, autocomplete loopback, or dead-end.
//!
//! ## Routing through conditionals
//!
//! When the graph walk encounters a Conditional node, the item's `arg` is evaluated
//! against the conditional's [`Condition`](super::graph::Condition) list (in order).
//! The first matching condition's `uid` identifies the output port to follow; if no
//! condition matches, the else branch is taken.

use super::graph::{Condition, ObjectKind, WorkflowGraph};

/// The outcome of actioning an item from a Script Filter screen.
///
/// Determined by walking the workflow graph from the source Script Filter,
/// following connections (optionally gated by modifier keys) through conditionals
/// to the final destination.
///
/// # Example
///
/// ```
/// use alfrusco::simulator::ActionResult;
///
/// let result = ActionResult::DrilledIn {
///     target_uid: "SF-SUB-001".to_string(),
/// };
/// assert_eq!(result.assert_drills_in(), "SF-SUB-001");
///
/// let result = ActionResult::OpenedUrl {
///     url_template: "{query}".to_string(),
/// };
/// assert_eq!(result.assert_opens_url(), "{query}");
///
/// let result = ActionResult::TypedAutocomplete {
///     text: "fruits".to_string(),
/// };
/// assert_eq!(result.assert_autocompletes(), "fruits");
///
/// assert!(ActionResult::DeadEnd.is_dead_end());
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

/// Context about the actioned item, used for conditional evaluation.
#[derive(Debug, Clone, Default)]
pub(crate) struct ItemContext<'a> {
    /// The item's `arg` value (used when a condition's `inputstring` is `{query}`).
    pub arg: Option<&'a str>,
    /// The item's variables (used when a condition references `{var:name}`).
    pub variables: Option<&'a serde_json::Value>,
}

/// Walks the graph from a source Script Filter to determine the action outcome.
///
/// The routing logic:
/// 1. If the item has `valid: false` and an `autocomplete` string, it's a loopback.
/// 2. Otherwise, follow outgoing connections from the source (filtered by modifier).
/// 3. Walk through Conditional nodes by evaluating conditions against the item's arg.
/// 4. The first non-conditional destination determines the result type.
pub(crate) fn resolve_action(
    graph: &WorkflowGraph,
    source_uid: &str,
    item_valid: bool,
    item_autocomplete: Option<&str>,
    item_context: &ItemContext<'_>,
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
    resolve_from_node(graph, source_uid, item_context, modifiers, &mut Vec::new())
}

/// Recursively resolves the action by walking through conditionals.
fn resolve_from_node(
    graph: &WorkflowGraph,
    uid: &str,
    item_context: &ItemContext<'_>,
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
            // Evaluate conditions to determine which branch to follow
            resolve_conditional(graph, dest_node, dest_uid, item_context, modifiers, visited)
        }
        ObjectKind::CallExternalTrigger => {
            // Resolve to the matching External Trigger input and continue traversal
            resolve_external_trigger(graph, dest_node, item_context, modifiers, visited)
        }
        // Clipboard, ExternalTrigger (as direct destination), other — treat as action endpoint
        ObjectKind::Clipboard
        | ObjectKind::Other(_)
        | ObjectKind::Keyword
        | ObjectKind::ExternalTrigger => ActionResult::RanScript {
            target_uid: dest_uid.clone(),
        },
    }
}

/// Evaluates a Conditional node's conditions against the item context and follows
/// the matching branch.
///
/// For each condition (in order), resolves the `inputstring` to an actual value
/// (from `{query}` = item arg, or `{var:name}` = item variable), then evaluates
/// the match mode. The first matching condition's output UID determines which
/// connection to follow. If no condition matches, the else branch is taken.
fn resolve_conditional(
    graph: &WorkflowGraph,
    cond_node: &super::graph::ObjectNode,
    cond_uid: &str,
    item_context: &ItemContext<'_>,
    modifiers: u64,
    visited: &mut Vec<String>,
) -> ActionResult {
    let conditions = &cond_node.conditions;
    let connections = graph.outgoing_connections(cond_uid, Some(modifiers));
    let connections = if connections.is_empty() && modifiers != 0 {
        graph.outgoing_connections(cond_uid, Some(0))
    } else {
        connections
    };

    if connections.is_empty() {
        return ActionResult::DeadEnd;
    }

    // If the conditional has no conditions parsed (legacy/simple fixture),
    // or no connections have sourceoutputuid, fall through transparently.
    let has_output_routing = connections.iter().any(|c| c.source_output_uid.is_some());
    if conditions.is_empty() || !has_output_routing {
        // Legacy behavior: follow the first connection transparently
        let dest_uid = &connections[0].destination_uid;
        let Some(dest_node) = graph.objects().get(dest_uid) else {
            return ActionResult::DeadEnd;
        };
        return classify_or_continue(graph, dest_node, dest_uid, item_context, modifiers, visited);
    }

    // Evaluate conditions in order
    if let Some(matched_uid) = evaluate_conditions(conditions, item_context) {
        // Find the connection whose sourceoutputuid matches
        if let Some(conn) = connections
            .iter()
            .find(|c| c.source_output_uid.as_deref() == Some(matched_uid))
        {
            let dest_uid = &conn.destination_uid;
            let Some(dest_node) = graph.objects().get(dest_uid) else {
                return ActionResult::DeadEnd;
            };
            return classify_or_continue(
                graph,
                dest_node,
                dest_uid,
                item_context,
                modifiers,
                visited,
            );
        }
    }

    // No condition matched → take the else branch.
    // The else connection is the one whose sourceoutputuid doesn't match any condition uid.
    let condition_uids: Vec<Option<&str>> = conditions.iter().map(|c| c.uid.as_deref()).collect();
    let else_conn = connections.iter().find(|c| {
        let output = c.source_output_uid.as_deref();
        // Else branch: has no sourceoutputuid, or its uid isn't in the conditions list
        output.is_none() || !condition_uids.contains(&output)
    });

    if let Some(conn) = else_conn {
        let dest_uid = &conn.destination_uid;
        let Some(dest_node) = graph.objects().get(dest_uid) else {
            return ActionResult::DeadEnd;
        };
        classify_or_continue(graph, dest_node, dest_uid, item_context, modifiers, visited)
    } else {
        // No else connection and no condition matched → dead-end
        ActionResult::DeadEnd
    }
}

/// Classifies a destination node or continues walking if it's another conditional.
fn classify_or_continue(
    graph: &WorkflowGraph,
    dest_node: &super::graph::ObjectNode,
    dest_uid: &str,
    item_context: &ItemContext<'_>,
    modifiers: u64,
    visited: &mut Vec<String>,
) -> ActionResult {
    match &dest_node.kind {
        ObjectKind::ScriptFilter => ActionResult::DrilledIn {
            target_uid: dest_uid.to_string(),
        },
        ObjectKind::OpenUrl => {
            let url_template = dest_node
                .config_value("url")
                .unwrap_or_default()
                .to_string();
            ActionResult::OpenedUrl { url_template }
        }
        ObjectKind::RunScript => ActionResult::RanScript {
            target_uid: dest_uid.to_string(),
        },
        ObjectKind::Conditional => {
            resolve_conditional(graph, dest_node, dest_uid, item_context, modifiers, visited)
        }
        ObjectKind::CallExternalTrigger => {
            // Resolve to the matching External Trigger input and continue traversal
            resolve_external_trigger(graph, dest_node, item_context, modifiers, visited)
        }
        ObjectKind::Clipboard
        | ObjectKind::Other(_)
        | ObjectKind::Keyword
        | ObjectKind::ExternalTrigger => ActionResult::RanScript {
            target_uid: dest_uid.to_string(),
        },
    }
}

/// Resolves a `CallExternalTrigger` node by finding the matching External Trigger
/// input (by trigger ID) and continuing traversal from its outgoing connections.
///
/// This implements the `item → callexternaltrigger → external-trigger-input → …`
/// chain that Alfred uses for indirect navigation (e.g. drill-in via triggers).
fn resolve_external_trigger(
    graph: &WorkflowGraph,
    call_node: &super::graph::ObjectNode,
    item_context: &ItemContext<'_>,
    modifiers: u64,
    visited: &mut Vec<String>,
) -> ActionResult {
    let Some(trigger_id) = call_node.config_value("triggerid") else {
        return ActionResult::DeadEnd;
    };

    let Some(trigger_uid) = graph.external_trigger_uid(trigger_id) else {
        // No matching trigger input found → dead-end
        return ActionResult::DeadEnd;
    };

    // Continue traversal from the External Trigger input's outgoing connections
    resolve_from_node(graph, trigger_uid, item_context, modifiers, visited)
}

/// Evaluates conditions in order and returns the UID of the first matching condition.
fn evaluate_conditions<'a>(
    conditions: &'a [Condition],
    item_context: &ItemContext<'_>,
) -> Option<&'a str> {
    for condition in conditions {
        let input_value = resolve_input_string(&condition.input_string, item_context);
        if condition.match_mode.evaluate(
            &input_value,
            &condition.match_string,
            condition.match_case_sensitive,
        ) {
            return condition.uid.as_deref();
        }
    }
    None
}

/// Resolves a condition's `inputstring` to an actual value from the item context.
///
/// - `{query}` → item's arg
/// - `{var:name}` → item's variable named `name`
/// - Anything else → literal string (uncommon but possible)
fn resolve_input_string(input_string: &str, item_context: &ItemContext<'_>) -> String {
    if input_string == "{query}" {
        return item_context.arg.unwrap_or("").to_string();
    }
    if let Some(var_name) = input_string
        .strip_prefix("{var:")
        .and_then(|s| s.strip_suffix('}'))
    {
        if let Some(vars) = item_context.variables {
            if let Some(val) = vars.get(var_name).and_then(|v| v.as_str()) {
                return val.to_string();
            }
        }
        return String::new();
    }
    // Literal string (pass through)
    input_string.to_string()
}
