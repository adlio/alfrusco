//! Events, the triggers that fire them and the actions they run.

use serde::Serialize;
use serde_json::{Map, Value};

use super::property::{arg_to_value, one_or_many, push_arg};
use super::JsonAction;
use crate::Arg;

/// An event handler on a component: when one of its triggers fires, its
/// actions run, after an optional confirmation dialog.
///
/// ```
/// use alfrusco::ui::{Action, Confirm, Event, JsonAction, Trigger};
///
/// let event = Event::new(Trigger::Actioned)
///     .confirm(Confirm::new("Clear the list?"))
///     .action(Action::script("scripts/clear.sh"))
///     .action(JsonAction::new().sound("Pop"));
/// # let _ = event;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/events/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    #[serde(serialize_with = "one_or_many")]
    triggers: Vec<Trigger>,
    #[serde(serialize_with = "one_or_many")]
    action: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirm: Option<Confirm>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Event {
    /// An event fired by `trigger`. Add its actions with [`Event::action`].
    pub fn new(trigger: Trigger) -> Self {
        Event {
            triggers: vec![trigger],
            action: Vec::new(),
            confirm: None,
            extra: Map::new(),
        }
    }

    /// Adds another trigger that fires the same actions.
    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Adds an action. Several actions run in order as one compound action.
    pub fn action(mut self, action: impl Into<Action>) -> Self {
        self.action.push(action.into());
        self
    }

    /// Asks the user to confirm before the actions run.
    pub fn confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = Some(confirm);
        self
    }

    extra_methods!();
}

/// What fires an [`Event`]. Each component responds only to the triggers
/// that make sense for it.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Trigger {
    /// The UI is built but not yet shown.
    Preload,
    /// The UI has launched.
    Launched,
    /// The UI is about to quit.
    Quit,
    /// The component was actioned, such as a button pressed or return pressed
    /// in a text field.
    Actioned,
    /// The component's value or selection changed.
    Changed,
    /// The user stopped typing in a text component.
    TypingStopped,
    /// The mouse button went down on the component.
    MouseDown,
    /// The left mouse button went up on the component.
    MouseUp,
    /// The component was clicked.
    Clicked,
    /// The component was double-clicked.
    DoubleClicked,
}

/// A confirmation dialog shown before an event's actions run.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Confirm {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    iconpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    informativetext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmbuttontext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cancelbuttontext: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Confirm {
    /// A dialog with `message` as its main text.
    pub fn new(message: impl Into<String>) -> Self {
        Confirm {
            message: message.into(),
            iconpath: None,
            informativetext: None,
            confirmbuttontext: None,
            cancelbuttontext: None,
            extra: Map::new(),
        }
    }

    /// The path to the dialog's icon.
    pub fn icon_path(mut self, path: impl Into<String>) -> Self {
        self.iconpath = Some(path.into());
        self
    }

    /// Secondary text under the message.
    pub fn informative_text(mut self, text: impl Into<String>) -> Self {
        self.informativetext = Some(text.into());
        self
    }

    /// The confirm button's text. Alfred's default is "Confirm".
    pub fn confirm_button_text(mut self, text: impl Into<String>) -> Self {
        self.confirmbuttontext = Some(text.into());
        self
    }

    /// The cancel button's text. Alfred's default is "Cancel".
    pub fn cancel_button_text(mut self, text: impl Into<String>) -> Self {
        self.cancelbuttontext = Some(text.into());
        self
    }

    extra_methods!();
}

/// One action run by an event, a shortcut or a menu item.
///
/// `args` and `variables` may contain `{value:id.property}`, `{data:property}`
/// and `{var:name}` placeholders, which Alfred resolves when the action runs.
///
/// ```
/// use alfrusco::ui::{function, Action, Placeholder};
///
/// let run = Action::script("scripts/search.sh")
///     .arg(Placeholder::value("query", "text"))
///     .variable("mode", "fast")
///     .asynchronous(true);
/// let next = Action::function("results", function::SELECT_NEXT);
/// # let _ = (run, next);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/actions/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Action {
    #[serde(rename = "type")]
    kind: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<ActionSrc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Value>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    variables: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flattenarrayargs: Option<bool>,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    run_async: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum ActionSrc {
    Path(String),
    Json(Box<JsonAction>),
}

/// The kind of an [`Action`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    /// Write the args to the UI log at the info level.
    Log,
    /// Run a script and process its JSON output as JSON actions.
    Script,
    /// Process JSON actions.
    Json,
    /// Run functions on a component.
    Function,
}

impl Action {
    fn of(kind: ActionType, src: Option<ActionSrc>) -> Self {
        Action {
            kind,
            src,
            args: None,
            variables: Map::new(),
            flattenarrayargs: None,
            run_async: None,
            extra: Map::new(),
        }
    }

    /// Logs `message` at the info level.
    pub fn log(message: impl Into<Arg>) -> Self {
        Self::of(ActionType::Log, None).args(message)
    }

    /// Runs the script at `path`, relative to the `.alfredui` folder. Alfred
    /// processes the script's JSON output as JSON actions.
    pub fn script(path: impl Into<String>) -> Self {
        Self::of(ActionType::Script, Some(ActionSrc::Path(path.into())))
    }

    /// Processes `actions`.
    pub fn json(actions: JsonAction) -> Self {
        Self::of(ActionType::Json, Some(ActionSrc::Json(Box::new(actions))))
    }

    /// Processes the JSON actions in the file at `path`.
    pub fn json_file(path: impl Into<String>) -> Self {
        Self::of(ActionType::Json, Some(ActionSrc::Path(path.into())))
    }

    /// Runs one or more functions, such as [`function::SELECT_NEXT`], on the
    /// component with id `target`.
    pub fn function(target: impl Into<String>, functions: impl Into<Arg>) -> Self {
        Self::of(ActionType::Function, Some(ActionSrc::Path(target.into()))).args(functions)
    }

    /// Replaces the arguments: script arguments, function names, or the text a
    /// log action writes. A single string is one argument.
    pub fn args(mut self, args: impl Into<Arg>) -> Self {
        self.args = Some(arg_to_value(args.into()));
        self
    }

    /// Adds one argument after any already set. It may be any JSON value:
    /// Alfred passes booleans as `1` / `0`, numbers in decimal, `null` as an
    /// empty string and objects and arrays as compact JSON (observed in
    /// Alfred 6.0.3). With [`Action::flatten_array_args`], arrays become
    /// separate arguments instead.
    pub fn arg(mut self, arg: impl Into<Value>) -> Self {
        push_arg(&mut self.args, arg.into());
        self
    }

    /// Sets a variable before the action runs. Values may use `{value:}`
    /// placeholders, resolved when the action runs.
    ///
    /// Not in a script's environment: pass it with `{var:name}` in args, or
    /// declare it with [`Ui::variable_config`](super::Ui::variable_config).
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// When true, arrays among the args, including `{var:}` values that hold
    /// arrays, are flattened recursively into separate arguments.
    pub fn flatten_array_args(mut self, flatten: bool) -> Self {
        self.flattenarrayargs = Some(flatten);
        self
    }

    /// When true, the action runs in the background instead of blocking the
    /// UI, and its output is processed when it returns.
    #[doc(alias = "async")]
    pub fn asynchronous(mut self, run_async: bool) -> Self {
        self.run_async = Some(run_async);
        self
    }

    extra_methods!();
}

impl From<JsonAction> for Action {
    fn from(actions: JsonAction) -> Self {
        Action::json(actions)
    }
}

/// Names of the functions components support, for [`Action::function`] and
/// [`Functions`](super::Functions).
pub mod function {
    /// Window: close it.
    pub const CLOSE: &str = "close";
    /// Window: zoom (maximise) it.
    pub const ZOOM: &str = "zoom";
    /// Window: minimise it.
    pub const MINIMISE: &str = "minimise";

    /// Picker and dataview: select the first item.
    pub const SELECT_FIRST: &str = "selectfirst";
    /// Picker and dataview: select the last item.
    pub const SELECT_LAST: &str = "selectlast";
    /// Picker and dataview: select the previous item.
    pub const SELECT_PREVIOUS: &str = "selectprevious";
    /// Picker and dataview: select the next item.
    pub const SELECT_NEXT: &str = "selectnext";

    /// Dataview: filter the data again with its configured filter.
    pub const FILTER_DATA: &str = "filterdata";
    /// Dataview: select the item above.
    pub const SELECT_UP: &str = "selectup";
    /// Dataview: select the item below.
    pub const SELECT_DOWN: &str = "selectdown";
    /// Dataview: select the item to the left.
    pub const SELECT_LEFT: &str = "selectleft";
    /// Dataview: select the item to the right.
    pub const SELECT_RIGHT: &str = "selectright";
    /// Dataview: clear the selection.
    pub const SELECT_NONE: &str = "selectnone";

    /// Textarea and dataview: scroll to the top.
    pub const SCROLL_TOP: &str = "scrolltop";
    /// Textarea and dataview: scroll to the bottom.
    pub const SCROLL_BOTTOM: &str = "scrollbottom";
    /// Textarea: move the caret to the start of the text.
    pub const CARET_START: &str = "caretstart";
    /// Textarea: move the caret to the end of the text.
    pub const CARET_END: &str = "caretend";

    /// Webkitview: go back.
    pub const BACK: &str = "back";
    /// Webkitview: go forward.
    pub const FORWARD: &str = "forward";
    /// Webkitview: reload the page.
    pub const RELOAD: &str = "reload";
}
