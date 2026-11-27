//! Top-level settings: variables, the pulse, logging and debugging.

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

use crate::Arg;

/// How a variable behaves: its default, whether it persists or reaches
/// scripts, and where its first value comes from.
///
/// ```
/// use alfrusco::ui::{VariableConfig, VariableSource};
///
/// let query = VariableConfig::new().src(VariableSource::Arg(0)).script_env(true);
/// let theme = VariableConfig::new().default_value("light").persist(true);
/// # let _ = (query, theme);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/variables/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct VariableConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    persist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scriptenv: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<VariableSource>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl VariableConfig {
    /// Alfred's defaults: no default value, not persisted, and `scriptenv`
    /// off.
    pub fn new() -> Self {
        Self::default()
    }

    /// The value used when the variable is null, and the baseline persistence
    /// compares against.
    ///
    /// Alfred's docs list `default` as a string, but Alfred 6.0.3 also accepts
    /// numbers, booleans and objects, keeps their type, and persists numbers
    /// as numbers (observed).
    pub fn default_value(mut self, value: impl Into<Value>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Saves the value when it differs from the default and restores it at
    /// start-up. Needs [`Ui::bundle_id`](super::Ui::bundle_id).
    pub fn persist(mut self, persist: bool) -> Self {
        self.persist = Some(persist);
        self
    }

    /// Alfred's docs say this passes the variable to the environment of action
    /// scripts and the pulse.
    ///
    /// In Alfred 6.0.3 every `variablesconfig` key reaches the script
    /// environment whatever this says (observed with `scriptenv: false`), so
    /// don't rely on it either way. Variables not declared in
    /// `variablesconfig` never reach the environment.
    pub fn script_env(mut self, script_env: bool) -> Self {
        self.scriptenv = Some(script_env);
        self
    }

    /// Where the first value comes from.
    pub fn src(mut self, src: VariableSource) -> Self {
        self.src = Some(src);
        self
    }

    extra_methods!();
}

/// Where a variable's first value comes from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableSource {
    /// The launch argument at this index: `args[n]`.
    Arg(usize),
    /// Every launch argument, as an array: `args`.
    Args,
    /// The workflow variable with the same name: `variable`. Alfred UI only
    /// sees workflow variables configured this way.
    Variable,
}

impl Serialize for VariableSource {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            VariableSource::Arg(index) => serializer.serialize_str(&format!("args[{index}]")),
            VariableSource::Args => serializer.serialize_str("args"),
            VariableSource::Variable => serializer.serialize_str("variable"),
        }
    }
}

/// A script Alfred runs on a timer while the UI is open. Its JSON output is
/// processed as JSON actions, which may reconfigure the pulse.
///
/// ```
/// use alfrusco::ui::{IntervalMode, Pulse};
///
/// let pulse = Pulse::new("scripts/poll.sh").interval(2.0).interval_mode(IntervalMode::Regular);
/// let stop = Pulse::stop();
/// # let _ = (pulse, stop);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/pulse/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Pulse {
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Arg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    now: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intervalmode: Option<IntervalMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<f64>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Pulse {
    /// A pulse that runs the script at `src`, relative to the `.alfredui`
    /// folder. Set an interval to start it.
    pub fn new(src: impl Into<String>) -> Self {
        Pulse {
            src: Some(src.into()),
            ..Self::default()
        }
    }

    /// A change to the running pulse with no `src`, for a JSON action.
    pub fn change() -> Self {
        Self::default()
    }

    /// Turns the pulse off: interval 0.
    pub fn stop() -> Self {
        Self::change().interval(0.0)
    }

    /// Script arguments: plain strings or whole-string `{var:name}`
    /// placeholders, which Alfred resolves again on every tick. UI values
    /// (`{value:}`) and composite strings such as `"x-{var:a}"` can't be used.
    pub fn args(mut self, args: impl Into<Arg>) -> Self {
        self.args = Some(args.into());
        self
    }

    /// When true, runs one extra pulse right after the current event.
    pub fn now(mut self, now: bool) -> Self {
        self.now = Some(now);
        self
    }

    /// How the interval is measured.
    pub fn interval_mode(mut self, mode: IntervalMode) -> Self {
        self.intervalmode = Some(mode);
        self
    }

    /// Seconds between pulses, 0.1-1000. 0 turns the pulse off.
    pub fn interval(mut self, seconds: f64) -> Self {
        self.interval = Some(seconds);
        self
    }

    extra_methods!();
}

/// How a [`Pulse`] interval is measured.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IntervalMode {
    /// Wait the interval after the previous run finishes. This is the default.
    Delay,
    /// Fire as close to every interval as possible.
    Regular,
}

/// Where and how much the UI logs.
///
/// See <https://www.alfredapp.com/help/alfredui/log/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Log {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<LogLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colourful: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    console: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    internal: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Log {
    /// Logs at `level`.
    pub fn new(level: LogLevel) -> Self {
        Log {
            level: Some(level),
            ..Self::default()
        }
    }

    /// The log file. Use an absolute path: that is what was observed writing
    /// a file in Alfred 6.0.3. Without a path, nothing reached disk.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Prefixes entries with a timestamp.
    pub fn timestamp(mut self, timestamp: bool) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Writes ANSI colours to the log file.
    pub fn colourful(mut self, colourful: bool) -> Self {
        self.colourful = Some(colourful);
        self
    }

    /// Also logs to the macOS console.
    pub fn console(mut self, console: bool) -> Self {
        self.console = Some(console);
        self
    }

    /// Also logs to the workflow debugger, when the UI runs inside Alfred.
    pub fn internal(mut self, internal: bool) -> Self {
        self.internal = Some(internal);
        self
    }

    extra_methods!();
}

/// How much a [`Log`] records. Each level includes the ones before it.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum LogLevel {
    /// Nothing. This is the default.
    None,
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
    /// Everything.
    All,
}

/// Layout debugging aids.
///
/// See <https://www.alfredapp.com/help/alfredui/debug/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DebugSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    colourful: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reloadshortcut: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl DebugSettings {
    /// No debugging aids.
    pub fn new() -> Self {
        Self::default()
    }

    /// Colours views randomly to show the layout.
    pub fn colourful(mut self, colourful: bool) -> Self {
        self.colourful = Some(colourful);
        self
    }

    /// A shortcut such as `"⌘R"` that reloads a UI running outside Alfred.
    pub fn reload_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.reloadshortcut = Some(shortcut.into());
        self
    }

    extra_methods!();
}
