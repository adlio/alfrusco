//! Checks Alfred UI doesn't do itself: it ignores unknown keys, unknown ids
//! and missing scripts without telling the user, so these find them first.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value};

use super::{Ui, UiContext};
use crate::simulator::Severity;
use crate::{Error, Result};

/// The JSON action commands Alfred 6 understands, in the order it runs them.
pub const JSON_ACTION_COMMANDS: &[&str] = &[
    "variables",
    "pulse",
    "remove",
    "insert",
    "update",
    "replace",
    "sheet",
    "share",
    "sound",
    "speak",
    "copy",
    "open",
    "opendialog",
    "window",
    "notification",
    "functions",
    "popupmenu",
    "finish",
];

const UI_JSON_KEYS: &[&str] = &[
    "bundleid",
    "window",
    "variables",
    "variablesconfig",
    "templates",
    "colours",
    "pulse",
    "log",
    "debug",
    "iconpath",
];

const COMPONENT_TYPES: &[&str] = &[
    "view",
    "label",
    "button",
    "checkbox",
    "picker",
    "textfield",
    "passwordfield",
    "textarea",
    "markdownarea",
    "progress",
    "imageview",
    "dataview",
    "webkitview",
    "template",
    "file",
];

const ACTION_TYPES: &[&str] = &["log", "script", "json", "function"];

static VALUE_PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{value:([^.}]+)[.}]").expect("valid regex"));
static PARAM_PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{param:([^}]+)\}").expect("valid regex"));

/// One finding from a check.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// How sure the check is that this breaks the UI.
    pub severity: Severity,
    /// Where: a JSON path such as `/window/root/components/1`, prefixed with
    /// a file name when the problem is in a component file.
    pub location: String,
    /// What's wrong.
    pub message: String,
}

impl Problem {
    fn new(severity: Severity, location: impl Into<String>, message: impl Into<String>) -> Self {
        Problem {
            severity,
            location: location.into(),
            message: message.into(),
        }
    }

    /// True for [`Severity::Error`].
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let location = if self.location.is_empty() {
            "/"
        } else {
            &self.location
        };
        write!(f, "{severity}: {location}: {}", self.message)
    }
}

/// Checks a JSON actions object, such as a script's output, for what Alfred
/// would silently ignore or drop.
///
/// - A key that isn't a JSON action command is ignored by Alfred without a
///   warning. Within two edits of a known command (`updat`, `spek`) it's an
///   error, since it's almost certainly a typo. Anything else is a warning,
///   because it may be a command Alfred added after this version of alfrusco,
///   written with `JsonAction::set`.
/// - A top-level array is a warning: Alfred accepts it and does nothing.
/// - `finish.variables` values that are objects or arrays are errors, because
///   Alfred drops them before the next object; booleans and `null` are
///   warnings, because that path wasn't tested.
///
/// ```
/// use alfrusco::ui::check_actions;
/// use serde_json::json;
///
/// let problems = check_actions(&json!({ "updat": { "ids": {} } }));
/// assert_eq!(problems.len(), 1);
/// assert!(problems[0].message.contains("updat"));
/// ```
pub fn check_actions(actions: &Value) -> Vec<Problem> {
    let mut problems = Vec::new();
    check_actions_at(actions, "", &mut problems);
    problems
}

fn check_actions_at(actions: &Value, at: &str, problems: &mut Vec<Problem>) {
    let object = match actions {
        Value::Object(object) => object,
        Value::Array(_) => {
            problems.push(Problem::new(
                Severity::Warning,
                at,
                "a JSON array is accepted by Alfred and does nothing; write an object",
            ));
            return;
        }
        _ => {
            problems.push(Problem::new(
                Severity::Error,
                at,
                "JSON actions must be an object",
            ));
            return;
        }
    };
    for (key, value) in object {
        let here = format!("{at}/{key}");
        if !JSON_ACTION_COMMANDS.contains(&key.as_str()) {
            problems.push(unknown_command(key, &here));
            continue;
        }
        match key.as_str() {
            "variables" | "update" | "insert" | "replace" | "functions" if !value.is_object() => {
                problems.push(Problem::new(Severity::Error, &here, "must be an object"));
            }
            "update" | "insert" | "replace" | "functions" => {
                if !value.get("ids").is_some_and(Value::is_object) {
                    problems.push(Problem::new(
                        Severity::Error,
                        &here,
                        "needs an `ids` object",
                    ));
                }
            }
            "pulse" => check_pulse(value, &here, problems),
            "finish" => check_finish(value, &here, problems),
            _ => {}
        }
    }
}

/// An unknown JSON action key. A near-miss of a known command (edit distance
/// 2 or less, such as `updat`) is an error, since it's almost certainly a
/// typo. Anything else is a warning: it may be a command Alfred added after
/// this version of alfrusco, sent with `JsonAction::set`.
fn unknown_command(key: &str, at: &str) -> Problem {
    let near = JSON_ACTION_COMMANDS
        .iter()
        .find(|command| edit_distance(key, command) <= 2);
    match near {
        Some(command) => Problem::new(
            Severity::Error,
            at,
            format!("`{key}` is not a JSON action command (did you mean `{command}`?); Alfred ignores it silently"),
        ),
        None => Problem::new(
            Severity::Warning,
            at,
            format!("`{key}` is not a JSON action command this version of alfrusco knows; Alfred ignores unknown keys silently"),
        ),
    }
}

/// Levenshtein distance, for spotting typos in short command names.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitute = previous[j] + usize::from(ca != *cb);
            current.push(substitute.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

fn check_pulse(pulse: &Value, at: &str, problems: &mut Vec<Problem>) {
    if let Some(interval) = pulse.get("interval").and_then(Value::as_f64) {
        if interval != 0.0 && !(0.1..=1000.0).contains(&interval) {
            problems.push(Problem::new(
                Severity::Warning,
                format!("{at}/interval"),
                "the pulse interval must be 0 (off) or 0.1-1000 seconds; Alfred caps it",
            ));
        }
    }
    // Alfred's pulse docs: args are plain strings or whole-string `{var:}`
    // placeholders; UI values and composite strings can't be used.
    let args = match pulse.get("args") {
        Some(Value::Array(args)) => args.iter().collect(),
        Some(arg) => vec![arg],
        None => Vec::new(),
    };
    for arg in args.into_iter().filter_map(Value::as_str) {
        let here = format!("{at}/args");
        if arg.contains("{value:") || arg.contains("{data:") {
            problems.push(Problem::new(
                Severity::Error,
                here,
                format!("pulse arg `{arg}` uses a UI value; the pulse can only use strings and `{{var:}}`"),
            ));
        } else if arg.contains("{var:") && whole_placeholder(arg, "var").is_none() {
            problems.push(Problem::new(
                Severity::Warning,
                here,
                format!("pulse arg `{arg}` mixes text and `{{var:}}`; Alfred's docs say the pulse can't use composite strings"),
            ));
        }
    }
}

fn check_finish(finish: &Value, at: &str, problems: &mut Vec<Problem>) {
    let Some(variables) = finish.get("variables").and_then(Value::as_object) else {
        return;
    };
    for (name, value) in variables {
        let here = format!("{at}/variables/{name}");
        match value {
            Value::String(_) | Value::Number(_) => {}
            Value::Object(_) | Value::Array(_) => problems.push(Problem::new(
                Severity::Error,
                here,
                "Alfred drops object and array finish variables; send a string or number",
            )),
            Value::Bool(_) | Value::Null => problems.push(Problem::new(
                Severity::Warning,
                here,
                "untested finish variable type; send a string or number",
            )),
        }
    }
}

/// An `.alfredui` folder: its `ui.json` with the component files it pulls in.
///
/// [`UiPackage::problems`] checks the whole UI for missing files, scripts
/// without the exec bit, ids that actions and `{value:}` placeholders target
/// but no component has, template parameters that are never given, and keys
/// and types Alfred doesn't know. [`UiPackage::run_script`] runs one of the
/// UI's scripts the way Alfred does and checks its output.
///
/// ```no_run
/// use alfrusco::ui::UiPackage;
///
/// let package = UiPackage::open("workflow/tasks.alfredui").unwrap();
/// let problems = package.problems();
/// assert!(problems.is_empty(), "{problems:#?}");
/// ```
#[derive(Debug, Clone)]
pub struct UiPackage {
    dir: PathBuf,
    json: Value,
    load_problems: Vec<Problem>,
}

impl UiPackage {
    /// Reads `<dir>/ui.json` and every component file it includes with a
    /// whole-string `{file:path}` placeholder.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        let path = dir.join("ui.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
        let json: Value = serde_json::from_str(&text)
            .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
        Ok(Self::from_json(dir, json))
    }

    /// A package from a typed [`Ui`] whose files live in `dir`.
    pub fn from_ui(dir: impl AsRef<Path>, ui: &Ui) -> Self {
        let json = serde_json::to_value(ui).unwrap_or(Value::Null);
        Self::from_json(dir.as_ref().to_path_buf(), json)
    }

    fn from_json(dir: PathBuf, json: Value) -> Self {
        let mut load_problems = Vec::new();
        let json = expand_files(&dir, json, "", 0, &mut load_problems);
        UiPackage {
            dir,
            json,
            load_problems,
        }
    }

    /// The `.alfredui` folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The UI JSON with `{file:}` component files inlined.
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// Every component id in the UI, including ids in templates and in
    /// components that actions insert or replace.
    pub fn ids(&self) -> BTreeSet<String> {
        let mut ids = BTreeMap::new();
        collect_ids(&self.json, "", &mut ids);
        ids.into_keys().collect()
    }

    /// Everything wrong with the UI that Alfred would not report.
    pub fn problems(&self) -> Vec<Problem> {
        let mut problems = self.load_problems.clone();
        let Some(root) = self.json.as_object() else {
            problems.push(Problem::new(
                Severity::Error,
                "",
                "ui.json must be an object",
            ));
            return problems;
        };
        for key in root.keys() {
            if !UI_JSON_KEYS.contains(&key.as_str()) {
                problems.push(Problem::new(
                    Severity::Warning,
                    format!("/{key}"),
                    format!("`{key}` is not a ui.json key this version of alfrusco knows"),
                ));
            }
        }
        if root.get("window").is_none_or(|w| w.get("root").is_none()) {
            problems.push(Problem::new(
                Severity::Error,
                "/window",
                "ui.json needs a window with a root component",
            ));
        }

        let mut ids = BTreeMap::new();
        collect_ids(&self.json, "", &mut ids);
        for (id, places) in &ids {
            let fixed: Vec<&String> = places.iter().filter(|p| !is_dynamic(p)).collect();
            if fixed.len() > 1 {
                problems.push(Problem::new(
                    Severity::Warning,
                    fixed[1].as_str(),
                    format!("id `{id}` is used more than once; ids should be unique"),
                ));
            }
        }
        let known: BTreeSet<&str> = ids.keys().map(String::as_str).collect();
        let templates = root.get("templates").and_then(Value::as_object);

        let mut walker = Walker {
            dir: &self.dir,
            known: &known,
            templates,
            problems: &mut problems,
            open_templates: Vec::new(),
        };
        walker.walk(&self.json, "", Context::Other);
        if let Some(pulse) = root.get("pulse") {
            if let Some(src) = pulse.get("src").and_then(Value::as_str) {
                walker.check_script(src, "/pulse/src");
            }
            check_pulse(pulse, "/pulse", walker.problems);
        }
        if let Some(icon) = root.get("iconpath").and_then(Value::as_str) {
            if !icon.contains('{') && !resolve(&self.dir, icon).exists() {
                walker.problems.push(Problem::new(
                    Severity::Warning,
                    "/iconpath",
                    format!("icon `{icon}` doesn't exist"),
                ));
            }
        }
        problems
    }

    /// Checks a response a script wrote: [`check_actions`], plus every id it
    /// targets must exist in this UI.
    pub fn check_response(&self, actions: &Value) -> Vec<Problem> {
        let mut problems = check_actions(actions);
        let mut ids = BTreeMap::new();
        collect_ids(&self.json, "", &mut ids);
        // Components the response inserts or replaces are new ids too.
        collect_ids(actions, "/action", &mut ids);
        let known: BTreeSet<&str> = ids.keys().map(String::as_str).collect();
        check_targets(actions, "", &known, &mut problems);
        problems
    }

    /// Runs the script `src` (relative to the `.alfredui` folder) the way
    /// Alfred 6 does, with `ctx`'s arguments and variables, and checks what it
    /// writes.
    ///
    /// The script is exec'd directly with the `.alfredui` folder as its
    /// working directory and stdin at EOF. Its environment is only `PATH`,
    /// `HOME`, `TMPDIR`, `USER`, `LOGNAME`, `LANG` and `LC_CTYPE` from this
    /// process plus
    /// `ctx`'s variables, so a script that depends on anything else fails
    /// here as it would for a user. Alfred's own process environment is
    /// larger; this is deliberately the smaller set.
    pub fn run_script(&self, src: &str, ctx: &UiContext) -> ScriptRun {
        let path = resolve(&self.dir, src);
        let mut command = Command::new(&path);
        command
            .args(ctx.args())
            .current_dir(&self.dir)
            .stdin(Stdio::null())
            .env_clear();
        for key in [
            "PATH", "HOME", "TMPDIR", "USER", "LOGNAME", "LANG", "LC_CTYPE",
        ] {
            if let Ok(value) = std::env::var(key) {
                command.env(key, value);
            }
        }
        command.envs(ctx.env());

        let output = match command.output() {
            Ok(output) => output,
            Err(error) => {
                return ScriptRun {
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    response: None,
                    problems: vec![Problem::new(
                        Severity::Error,
                        src,
                        format!("script launch failed ({error}); Alfred would log the error and skip it"),
                    )],
                };
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let mut problems = Vec::new();
        let response = match serde_json::from_str::<Value>(&stdout) {
            Ok(value) => Some(value),
            Err(error) => {
                let what = if stdout.trim().is_empty() {
                    "empty output".to_string()
                } else {
                    format!("output isn't exactly one JSON value ({error})")
                };
                problems.push(Problem::new(
                    Severity::Error,
                    src,
                    format!("{what}; Alfred would log \"Response is not valid JSON\" and drop it"),
                ));
                None
            }
        };
        if !output.status.success() {
            problems.push(Problem::new(
                Severity::Error,
                src,
                format!(
                    "exited with {}; Alfred drops the output of a failed script",
                    output.status
                ),
            ));
        }
        if let Some(response) = &response {
            problems.extend(self.check_response(response));
        }
        ScriptRun {
            exit_code: output.status.code(),
            stdout,
            stderr,
            response,
            problems,
        }
    }
}

/// The result of [`UiPackage::run_script`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptRun {
    /// The exit code, or `None` when the script didn't start or was killed by
    /// a signal.
    pub exit_code: Option<i32>,
    /// What the script wrote to stdout.
    pub stdout: String,
    /// What it wrote to stderr. Alfred logs each line and never parses it.
    pub stderr: String,
    /// The parsed JSON actions, when stdout was exactly one JSON value.
    pub response: Option<Value>,
    /// Everything that would make Alfred drop or ignore the output.
    pub problems: Vec<Problem>,
}

impl ScriptRun {
    /// True when Alfred would apply the output and nothing was ignored.
    pub fn is_ok(&self) -> bool {
        self.problems.iter().all(|p| !p.is_error())
    }
}

fn resolve(dir: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}

fn whole_placeholder<'a>(value: &'a str, kind: &str) -> Option<&'a str> {
    value
        .strip_prefix('{')?
        .strip_suffix('}')?
        .strip_prefix(kind)?
        .strip_prefix(':')
}

const MAX_FILE_DEPTH: usize = 16;

/// Inlines whole-string `{file:path}` placeholders and records missing
/// `{file:}` and `{fileraw:}` files.
fn expand_files(
    dir: &Path,
    value: Value,
    at: &str,
    depth: usize,
    problems: &mut Vec<Problem>,
) -> Value {
    match value {
        Value::String(text) => {
            if let Some(path) = whole_placeholder(&text, "fileraw") {
                if !resolve(dir, path).is_file() {
                    problems.push(Problem::new(
                        Severity::Error,
                        at,
                        format!("`{{fileraw:{path}}}` doesn't exist"),
                    ));
                }
                return Value::String(text);
            }
            let Some(path) = whole_placeholder(&text, "file") else {
                return Value::String(text);
            };
            if depth >= MAX_FILE_DEPTH {
                problems.push(Problem::new(
                    Severity::Error,
                    at,
                    format!("`{{file:{path}}}` nests too deep; is there a cycle?"),
                ));
                return Value::String(text);
            }
            let file = resolve(dir, path);
            let Ok(contents) = std::fs::read_to_string(&file) else {
                problems.push(Problem::new(
                    Severity::Error,
                    at,
                    format!("`{{file:{path}}}` doesn't exist"),
                ));
                return Value::String(text);
            };
            match serde_json::from_str::<Value>(&contents) {
                Ok(json) => expand_files(dir, json, &format!("{path}:"), depth + 1, problems),
                Err(error) => {
                    problems.push(Problem::new(
                        Severity::Warning,
                        at,
                        format!(
                            "`{{file:{path}}}` isn't a JSON value ({error}), so it wasn't checked"
                        ),
                    ));
                    Value::String(text)
                }
            }
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .enumerate()
                .map(|(i, item)| expand_files(dir, item, &format!("{at}/{i}"), depth, problems))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, item)| {
                    let here = format!("{at}/{key}");
                    (key, expand_files(dir, item, &here, depth, problems))
                })
                .collect(),
        ),
        other => other,
    }
}

/// True for places whose ids may legitimately repeat: template bodies and
/// components that actions insert or replace.
fn is_dynamic(location: &str) -> bool {
    location.starts_with("/templates") || location.contains("/action")
}

fn collect_ids(value: &Value, at: &str, ids: &mut BTreeMap<String, Vec<String>>) {
    match value {
        Value::Object(object) => {
            if let Some(id) = object.get("id").and_then(Value::as_str) {
                ids.entry(id.to_string()).or_default().push(at.to_string());
            }
            for (key, item) in object {
                // Keys of insert, update and replace `ids` maps aren't new ids,
                // but replacement components inside them may carry one.
                collect_ids(item, &format!("{at}/{key}"), ids);
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                collect_ids(item, &format!("{at}/{i}"), ids);
            }
        }
        _ => {}
    }
}

/// Checks the ids a JSON actions object targets.
fn check_targets(actions: &Value, at: &str, known: &BTreeSet<&str>, problems: &mut Vec<Problem>) {
    let Some(object) = actions.as_object() else {
        return;
    };
    let mut targets: Vec<(String, String)> = Vec::new();
    for command in ["update", "replace", "insert", "functions"] {
        if let Some(ids) = object
            .get(command)
            .and_then(|c| c.get("ids"))
            .and_then(Value::as_object)
        {
            for id in ids.keys() {
                targets.push((format!("{at}/{command}/ids/{id}"), id.clone()));
            }
        }
    }
    if let Some(ids) = object.get("remove").and_then(|r| r.get("ids")) {
        for id in string_or_array(ids) {
            targets.push((format!("{at}/remove/ids"), id));
        }
    }
    if let Some(id) = object
        .get("popupmenu")
        .and_then(|m| m.get("src"))
        .and_then(Value::as_str)
    {
        targets.push((format!("{at}/popupmenu/src"), id.to_string()));
    }
    for (location, id) in targets {
        if !id.contains('{') && !known.contains(id.as_str()) {
            problems.push(Problem::new(
                Severity::Error,
                location,
                format!("no component has id `{id}`; Alfred ignores the change"),
            ));
        }
    }
}

fn string_or_array(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Component,
    Action,
    Other,
}

struct Walker<'a> {
    dir: &'a Path,
    known: &'a BTreeSet<&'a str>,
    templates: Option<&'a Map<String, Value>>,
    problems: &'a mut Vec<Problem>,
    /// Template files being checked, to stop a template that uses itself.
    open_templates: Vec<String>,
}

impl Walker<'_> {
    fn walk(&mut self, value: &Value, at: &str, context: Context) {
        match value {
            Value::String(text) => self.check_string(text, at),
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    self.walk(item, &format!("{at}/{i}"), context);
                }
            }
            Value::Object(object) => {
                match context {
                    Context::Component => self.check_component(object, at),
                    Context::Action => self.check_action(object, at),
                    Context::Other => {}
                }
                for (key, item) in object {
                    let child = match key.as_str() {
                        "components" | "root" => Context::Component,
                        "action" => Context::Action,
                        _ if at == "/templates" => Context::Component,
                        _ => Context::Other,
                    };
                    self.walk(item, &format!("{at}/{key}"), child);
                }
            }
            _ => {}
        }
    }

    fn check_string(&mut self, text: &str, at: &str) {
        for capture in VALUE_PLACEHOLDER.captures_iter(text) {
            let id = &capture[1];
            if id != "self" && !id.contains('{') && !self.known.contains(id) {
                self.problems.push(Problem::new(
                    Severity::Error,
                    at,
                    format!("`{{value:{id}...}}` names no component; Alfred resolves it to an empty string"),
                ));
            }
        }
    }

    fn check_component(&mut self, component: &Map<String, Value>, at: &str) {
        let kind = component
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("view");
        if !COMPONENT_TYPES.contains(&kind) {
            self.problems.push(Problem::new(
                Severity::Warning,
                format!("{at}/type"),
                format!("`{kind}` is not a component type this version of alfrusco knows"),
            ));
        }
        let src = component.get("src").and_then(Value::as_str);
        match (kind, src) {
            ("template", Some(src)) => self.check_template(component, src, at),
            ("template", None) => self.problems.push(Problem::new(
                Severity::Error,
                at,
                "a template needs a `src`",
            )),
            ("file", Some(src)) if !src.contains('{') && !resolve(self.dir, src).is_file() => {
                self.problems.push(Problem::new(
                    Severity::Error,
                    format!("{at}/src"),
                    format!("component file `{src}` doesn't exist"),
                ));
            }
            _ => {}
        }
    }

    fn check_template(&mut self, component: &Map<String, Value>, src: &str, at: &str) {
        let body = match self.templates.and_then(|t| t.get(src)) {
            Some(body) => body.clone(),
            None => {
                let file = resolve(self.dir, src);
                let parsed = std::fs::read_to_string(&file)
                    .ok()
                    .and_then(|text| serde_json::from_str::<Value>(&text).ok());
                let Some(body) = parsed else {
                    self.problems.push(Problem::new(
                        Severity::Error,
                        format!("{at}/src"),
                        format!(
                            "template `{src}` is neither in `templates` nor a readable JSON file"
                        ),
                    ));
                    return;
                };
                if !self.open_templates.iter().any(|open| open == src) {
                    self.open_templates.push(src.to_string());
                    self.walk(&body, &format!("{src}:"), Context::Component);
                    self.open_templates.pop();
                }
                body
            }
        };
        let mut wanted = BTreeSet::new();
        collect_params(&body, &mut wanted);
        let given = component.get("params").and_then(Value::as_object);
        for param in wanted {
            if !given.is_some_and(|p| p.contains_key(&param)) {
                self.problems.push(Problem::new(
                    Severity::Warning,
                    at,
                    format!(
                        "template `{src}` uses `{{param:{param}}}` but this use doesn't set it"
                    ),
                ));
            }
        }
    }

    fn check_action(&mut self, action: &Map<String, Value>, at: &str) {
        let kind = action.get("type").and_then(Value::as_str).unwrap_or("");
        if !ACTION_TYPES.contains(&kind) {
            self.problems.push(Problem::new(
                Severity::Error,
                format!("{at}/type"),
                format!("`{kind}` is not an action type (log, script, json or function)"),
            ));
            return;
        }
        let src = action.get("src");
        match (kind, src) {
            ("script", Some(Value::String(src))) => self.check_script(src, &format!("{at}/src")),
            ("json", Some(Value::String(path))) if !path.contains('{') => {
                let file = resolve(self.dir, path);
                match std::fs::read_to_string(&file)
                    .ok()
                    .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                {
                    Some(actions) => {
                        check_actions_at(&actions, &format!("{path}:"), self.problems);
                        check_targets(&actions, &format!("{path}:"), self.known, self.problems);
                    }
                    None => self.problems.push(Problem::new(
                        Severity::Error,
                        format!("{at}/src"),
                        format!("JSON actions file `{path}` doesn't exist or isn't JSON"),
                    )),
                }
            }
            ("json", Some(actions @ Value::Object(_))) => {
                let here = format!("{at}/src");
                check_actions_at(actions, &here, self.problems);
                check_targets(actions, &here, self.known, self.problems);
            }
            ("function", Some(Value::String(id))) => {
                if !id.contains('{') && !self.known.contains(id.as_str()) {
                    self.problems.push(Problem::new(
                        Severity::Error,
                        format!("{at}/src"),
                        format!("no component has id `{id}`; the function call does nothing"),
                    ));
                }
            }
            ("log", _) => {}
            _ => self.problems.push(Problem::new(
                Severity::Error,
                format!("{at}/src"),
                format!("a `{kind}` action needs a `src`"),
            )),
        }
    }

    fn check_script(&mut self, src: &str, at: &str) {
        if src.contains('{') {
            return;
        }
        let path = resolve(self.dir, src);
        if !path.is_file() {
            self.problems.push(Problem::new(
                Severity::Error,
                at,
                format!("script `{src}` doesn't exist; Alfred logs \"Script launch failed\" and skips it"),
            ));
            return;
        }
        if !is_executable(&path) {
            self.problems.push(Problem::new(
                Severity::Error,
                at,
                format!("script `{src}` isn't executable; Alfred execs it directly, so it needs the exec bit and a shebang"),
            ));
        }
    }
}

fn collect_params(value: &Value, params: &mut BTreeSet<String>) {
    match value {
        Value::String(text) => {
            for capture in PARAM_PLACEHOLDER.captures_iter(text) {
                params.insert(capture[1].to_string());
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_params(item, params)),
        Value::Object(object) => object
            .values()
            .for_each(|item| collect_params(item, params)),
        _ => {}
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
