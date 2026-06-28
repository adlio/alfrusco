//! Rendered Script Filter output for inspection and assertion.

use serde_json::Value;
use std::sync::Arc;

use super::action::{resolve_action, ActionResult};
use super::graph::WorkflowGraph;

/// Context needed to resolve actions from a screen (the graph + which filter produced it).
#[derive(Debug, Clone)]
struct ActionContext {
    graph: Arc<WorkflowGraph>,
    source_uid: String,
}

/// A rendered Script Filter result screen.
///
/// `Screen` wraps the parsed JSON output from an Alfred workflow invocation,
/// providing read-only accessors for inspecting items and their properties.
/// When created by a [`Simulator`](super::Simulator), it also carries graph
/// context for action routing.
///
/// # Example
///
/// ```
/// use alfrusco::simulator::Screen;
///
/// let json = r#"{"items":[{"title":"Hello","arg":"world","valid":true}]}"#;
/// let screen = Screen::from_json(json).unwrap();
/// assert_eq!(screen.items().len(), 1);
/// assert_eq!(screen.items()[0].title(), "Hello");
/// assert_eq!(screen.items()[0].arg(), Some("world"));
/// assert!(screen.items()[0].is_valid());
/// ```
#[derive(Debug, Clone)]
pub struct Screen {
    items: Vec<ScreenItem>,
    raw: Value,
    context: Option<ActionContext>,
}

impl Screen {
    /// Parses a JSON string (Alfred Script Filter output) into a `Screen`.
    ///
    /// A `Screen` created this way has no graph context; [`Screen::action`] will
    /// return `None`. Use [`Simulator`](super::Simulator) to get action-routable screens.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid or missing the `items` array.
    pub fn from_json(json: &str) -> Result<Self, ScreenError> {
        let raw: Value = serde_json::from_str(json).map_err(ScreenError::InvalidJson)?;
        let items_array = raw
            .get("items")
            .and_then(Value::as_array)
            .ok_or(ScreenError::MissingItems)?;

        let items = items_array.iter().map(|v| ScreenItem(v.clone())).collect();
        Ok(Self {
            items,
            raw,
            context: None,
        })
    }

    /// Creates a screen with graph context for action routing.
    pub(crate) fn with_context(mut self, graph: Arc<WorkflowGraph>, source_uid: String) -> Self {
        self.context = Some(ActionContext { graph, source_uid });
        self
    }

    /// Returns the items on this screen.
    pub fn items(&self) -> &[ScreenItem] {
        &self.items
    }

    /// Returns the number of items on this screen.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if there are no items on this screen.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the raw JSON value of the entire response.
    pub fn raw_json(&self) -> &Value {
        &self.raw
    }

    /// Asserts that this screen renders at least one item.
    ///
    /// # Panics
    ///
    /// Panics if the screen has no items.
    pub fn assert_renders(&self) {
        assert!(
            !self.items.is_empty(),
            "Expected screen to render items, but it was empty"
        );
    }

    /// Determines the action result for the item at the given index.
    ///
    /// Walks the workflow graph from the source Script Filter to determine what
    /// would happen if the user actioned this item (with the given modifier bitmask,
    /// defaulting to 0 for no modifiers).
    ///
    /// Returns `None` if this screen has no graph context (created via
    /// [`Screen::from_json`] rather than a [`Simulator`](super::Simulator)).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn action(&self, index: usize) -> Option<ActionResult> {
        self.action_with_modifiers(index, 0)
    }

    /// Determines the action result for the first item on this screen.
    ///
    /// Convenience wrapper around [`Screen::action`] with index 0.
    ///
    /// Returns `None` if this screen has no graph context or is empty.
    pub fn action_first(&self) -> Option<ActionResult> {
        if self.items.is_empty() {
            return None;
        }
        self.action(0)
    }

    /// Determines the action result for the item at the given index with a
    /// specific modifier bitmask.
    ///
    /// Alfred modifier bitmasks: Cmd=1048576, Alt=524288, Ctrl=262144,
    /// Shift=131072, Fn=8388608. Combine with bitwise OR.
    ///
    /// Returns `None` if this screen has no graph context.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn action_with_modifiers(&self, index: usize, modifiers: u64) -> Option<ActionResult> {
        let ctx = self.context.as_ref()?;
        let item = &self.items[index];
        Some(resolve_action(
            &ctx.graph,
            &ctx.source_uid,
            item.is_valid(),
            item.autocomplete(),
            modifiers,
        ))
    }
}

/// A single item in a rendered Alfred screen.
///
/// Provides read-only accessors for inspecting item properties.
#[derive(Debug, Clone)]
pub struct ScreenItem(Value);

impl ScreenItem {
    /// Returns the item's title.
    pub fn title(&self) -> &str {
        self.0.get("title").and_then(Value::as_str).unwrap_or("")
    }

    /// Returns the item's subtitle, if set.
    pub fn subtitle(&self) -> Option<&str> {
        self.0.get("subtitle").and_then(Value::as_str)
    }

    /// Returns the item's argument, if set (single-value form).
    pub fn arg(&self) -> Option<&str> {
        self.0.get("arg").and_then(Value::as_str)
    }

    /// Returns the item's argument as a list (handles both single and array forms).
    pub fn args(&self) -> Vec<&str> {
        match self.0.get("arg") {
            Some(Value::String(s)) => vec![s.as_str()],
            Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        }
    }

    /// Returns `true` if the item is valid (actionable).
    ///
    /// Defaults to `true` when not explicitly set (Alfred's default).
    pub fn is_valid(&self) -> bool {
        self.0.get("valid").and_then(Value::as_bool).unwrap_or(true)
    }

    /// Returns the item's autocomplete string, if set.
    pub fn autocomplete(&self) -> Option<&str> {
        self.0.get("autocomplete").and_then(Value::as_str)
    }

    /// Returns the item's uid, if set.
    pub fn uid(&self) -> Option<&str> {
        self.0.get("uid").and_then(Value::as_str)
    }

    /// Returns a variable value by key, if set on this item.
    pub fn variable(&self, key: &str) -> Option<&str> {
        self.0
            .get("variables")
            .and_then(|v| v.get(key))
            .and_then(Value::as_str)
    }

    /// Returns the raw JSON value of this item.
    pub fn raw(&self) -> &Value {
        &self.0
    }
}

/// Errors from parsing a Screen.
#[derive(Debug, thiserror::Error)]
pub enum ScreenError {
    /// The JSON output was not valid.
    #[error("invalid JSON output: {0}")]
    InvalidJson(serde_json::Error),

    /// The JSON lacked an `items` array.
    #[error("response JSON missing 'items' array")]
    MissingItems,
}
