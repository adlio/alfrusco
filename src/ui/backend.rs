//! The script backend: what an Alfred UI `script` action or pulse script
//! receives, and the JSON actions it writes back. The observed Alfred 6.0.3
//! contract is documented on the `ui` module.

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{Data, Finish, JsonAction, Post, Problem, Pulse, Row};
use crate::{Arg, Error, Result};

/// The environment variable Alfred sets to the id of the component whose
/// event ran the script. Absent for the pulse.
pub const SOURCE_VAR: &str = "alfred_ui_source";

/// What started the script.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// A `script` action run by a component event. `source` is the id of the
    /// component whose event fired, or `None` when it has no id.
    Event {
        /// The component id, from `alfred_ui_source`.
        source: Option<String>,
    },
    /// The UI's pulse.
    Pulse,
}

/// Everything Alfred hands a UI script: its arguments and its environment.
///
/// [`UiContext::from_env`] reads the real process. In tests, build one with
/// [`UiContext::new`] and the `with_arg`, `with_var` and `with_source`
/// builders, then run a
/// [`UiRunnable`] with [`UiContext::run`].
///
/// ```
/// use alfrusco::ui::{Invocation, UiContext};
///
/// let ctx = UiContext::new()
///     .with_arg("save")
///     .with_arg("1")
///     .with_var("task_id", "T-42")
///     .with_source("save_title");
/// assert_eq!(ctx.arg(0), Some("save"));
/// assert_eq!(ctx.arg_bool(1), Some(true));
/// assert_eq!(ctx.var("task_id"), Some("T-42"));
/// assert_eq!(ctx.invocation(), Invocation::Event { source: Some("save_title".into()) });
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiContext {
    args: Vec<String>,
    env: HashMap<String, String>,
    dir: Option<PathBuf>,
}

impl UiContext {
    /// An event invocation from a component with no id, with no args and no
    /// variables.
    pub fn new() -> Self {
        let mut ctx = Self::default();
        ctx.env.insert(SOURCE_VAR.to_string(), String::new());
        ctx
    }

    /// The arguments, environment and working directory of this process.
    pub fn from_env() -> Self {
        UiContext {
            args: std::env::args().skip(1).collect(),
            env: std::env::vars().collect(),
            dir: std::env::current_dir().ok(),
        }
    }

    /// Adds an argument.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds several arguments.
    pub fn with_args<S: Into<String>>(mut self, args: impl IntoIterator<Item = S>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Sets an environment variable, such as a `variablesconfig` entry or an
    /// `alfred_*` value.
    pub fn with_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(name.into(), value.into());
        self
    }

    /// Marks this as an event from the component with id `id`.
    pub fn with_source(self, id: impl Into<String>) -> Self {
        self.with_var(SOURCE_VAR, id)
    }

    /// Marks this as a pulse invocation.
    pub fn as_pulse(mut self) -> Self {
        self.env.remove(SOURCE_VAR);
        self
    }

    /// Sets the working directory, normally the `.alfredui` folder.
    pub fn with_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir = Some(dir.into());
        self
    }

    /// What started the script.
    pub fn invocation(&self) -> Invocation {
        match self.env.get(SOURCE_VAR) {
            None => Invocation::Pulse,
            Some(id) if id.is_empty() => Invocation::Event { source: None },
            Some(id) => Invocation::Event {
                source: Some(id.clone()),
            },
        }
    }

    /// True when the pulse started the script.
    pub fn is_pulse(&self) -> bool {
        self.invocation() == Invocation::Pulse
    }

    /// Every argument, in order.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// The argument at `index`, counted from 0.
    pub fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(String::as_str)
    }

    /// The argument at `index` when Alfred resolved it: `None` when it's
    /// missing, empty, or a placeholder Alfred left as literal text.
    pub fn resolved_arg(&self, index: usize) -> Option<&str> {
        self.arg(index)
            .filter(|arg| !arg.is_empty() && !is_unresolved_placeholder(arg))
    }

    /// The argument at `index` as a boolean: `1` or `true`, `0` or `false`.
    pub fn arg_bool(&self, index: usize) -> Option<bool> {
        self.arg(index).and_then(parse_bool)
    }

    /// The argument at `index` parsed as JSON, for objects, arrays and
    /// numbers. Alfred doesn't keep object key order.
    pub fn arg_json<T: DeserializeOwned>(&self, index: usize) -> Result<T> {
        let arg = self
            .arg(index)
            .ok_or_else(|| Error::Workflow(format!("missing argument {index}")))?;
        Ok(serde_json::from_str(arg)?)
    }

    /// An environment variable. Alfred writes an unset or `null`
    /// `variablesconfig` value as an empty string.
    pub fn var(&self, name: &str) -> Option<&str> {
        self.env.get(name).map(String::as_str)
    }

    /// A variable as a boolean: `1` or `true`, `0` or `false`.
    pub fn var_bool(&self, name: &str) -> Option<bool> {
        self.var(name).and_then(parse_bool)
    }

    /// A variable parsed as JSON, for objects, arrays and numbers.
    pub fn var_json<T: DeserializeOwned>(&self, name: &str) -> Result<T> {
        let value = self
            .var(name)
            .ok_or_else(|| Error::Workflow(format!("missing variable {name}")))?;
        Ok(serde_json::from_str(value)?)
    }

    /// The working directory: the `.alfredui` folder when Alfred runs the
    /// script.
    pub fn ui_dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// The workflow's bundle id, from `alfred_workflow_bundleid`.
    pub fn bundle_id(&self) -> Option<&str> {
        self.var("alfred_workflow_bundleid")
    }

    /// The running Alfred version, from `alfred_version`.
    pub fn alfred_version(&self) -> Option<&str> {
        self.var("alfred_version")
    }

    /// The workflow's data folder, created if needed. Alfred names it in
    /// `alfred_workflow_data` but doesn't create it.
    pub fn data_dir(&self) -> Result<PathBuf> {
        self.ensure_dir("alfred_workflow_data")
    }

    /// The workflow's cache folder, created if needed. Alfred names it in
    /// `alfred_workflow_cache` but doesn't create it.
    pub fn cache_dir(&self) -> Result<PathBuf> {
        self.ensure_dir("alfred_workflow_cache")
    }

    fn ensure_dir(&self, var: &str) -> Result<PathBuf> {
        let dir = self
            .var(var)
            .filter(|dir| !dir.is_empty())
            .ok_or_else(|| Error::MissingEnvVar(var.to_string()))?;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// The variables passed to a subprocess, for [`UiPackage::run_script`](super::UiPackage::run_script).
    pub(crate) fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Runs `runnable` in-process and returns the response it would write,
    /// with failures turned into an error notification as [`execute_ui`] does.
    pub fn run<R: UiRunnable>(&self, runnable: R) -> UiResponse {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut response = UiResponse::new();
            match runnable.run(self, &mut response) {
                Ok(()) => response,
                Err(error) => {
                    eprintln!("alfrusco: {error}");
                    R::error_response(&error, self)
                }
            }
        }));
        outcome.unwrap_or_else(|payload| panic_response(payload.as_ref(), self))
    }

    /// The async form of [`UiContext::run`].
    pub async fn run_async<R: AsyncUiRunnable>(&self, runnable: R) -> UiResponse {
        let future = async {
            let mut response = UiResponse::new();
            match runnable.run_async(self, &mut response).await {
                Ok(()) => response,
                Err(error) => {
                    eprintln!("alfrusco: {error}");
                    R::error_response(&error, self)
                }
            }
        };
        let future = CatchUnwind {
            future: Box::pin(future),
        };
        match future.await {
            Ok(response) => response,
            Err(payload) => panic_response(payload.as_ref(), self),
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

/// True when `value` is a whole placeholder such as `{value:list.nosuch}`.
///
/// Alfred 6.0.3 replaces a missing `{var:}`, a missing component in
/// `{value:}` and a `{data:}` outside a cell with an empty string, but leaves
/// `{value:id.property}` as literal text when the component exists and lacks
/// the property.
pub fn is_unresolved_placeholder(value: &str) -> bool {
    let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) else {
        return false;
    };
    ["value:", "var:", "data:", "const:", "param:"]
        .iter()
        .any(|prefix| inner.starts_with(prefix))
}

/// The JSON actions a UI script writes back: variables, component updates,
/// notifications, pulse changes and `finish`.
///
/// ```
/// use alfrusco::ui::{Finish, Row, UiResponse};
///
/// let mut response = UiResponse::new();
/// response
///     .variable("token", "abc123")
///     .update("status", "text", "Saved")
///     .update("save", "enabled", false)
///     .rows("comments", [Row::new().uid("c1").value("body", "text", "Looks good")])
///     .notify("Task updated", "Title changed");
/// let json = response.to_json();
/// assert_eq!(json["update"]["ids"]["status"]["text"], "Saved");
/// assert_eq!(json["update"]["ids"]["comments"]["data"]["src"][0]["uid"], "c1");
/// assert!(response.problems().is_empty());
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiResponse {
    actions: JsonAction,
}

impl UiResponse {
    /// A response that changes nothing. It writes `{}`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a UI variable. Values keep their JSON type for later `{var:}`
    /// use.
    pub fn variable(&mut self, name: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.actions.variables.insert(name.into(), value.into());
        self
    }

    /// Resets a variable to its `variablesconfig` default by setting it to
    /// `null`.
    pub fn reset_variable(&mut self, name: impl Into<String>) -> &mut Self {
        self.variable(name, Value::Null)
    }

    /// Sets `property` on the component with id `id`.
    pub fn update(
        &mut self,
        id: impl Into<String>,
        property: impl Into<String>,
        value: impl Into<Value>,
    ) -> &mut Self {
        let update = self.actions.update.take().unwrap_or_default();
        self.actions.update = Some(update.property(id, property, value));
        self
    }

    /// Asks Alfred to lay out the UI again after the updates, for updates that
    /// change a component's size.
    pub fn relayout(&mut self) -> &mut Self {
        let update = self.actions.update.take().unwrap_or_default();
        self.actions.update = Some(update.relayout(true));
        self
    }

    /// Replaces the rows of the dataview with id `id`.
    pub fn rows(
        &mut self,
        id: impl Into<String>,
        rows: impl IntoIterator<Item = Row>,
    ) -> &mut Self {
        self.update(id, "data", Data::rows(rows))
    }

    /// Posts a notification.
    pub fn notify(&mut self, title: impl Into<String>, text: impl Into<String>) -> &mut Self {
        let notification = self.actions.notification.take().unwrap_or_default();
        self.actions.notification = Some(notification.post(Post::new(title, text)));
        self
    }

    /// Runs functions, such as [`function::SELECT_FIRST`](super::function::SELECT_FIRST),
    /// on the component with id `id`.
    pub fn call(&mut self, id: impl Into<String>, functions: impl Into<Arg>) -> &mut Self {
        let calls = self.actions.functions.take().unwrap_or_default();
        self.actions.functions = Some(calls.call(id, functions));
        self
    }

    /// Reconfigures the pulse.
    pub fn pulse(&mut self, pulse: Pulse) -> &mut Self {
        self.actions.pulse = Some(pulse);
        self
    }

    /// Stops the pulse.
    pub fn stop_pulse(&mut self) -> &mut Self {
        self.pulse(Pulse::stop())
    }

    /// Closes the UI. Inside Alfred (the object's launch mode 0) the args and
    /// string or number variables go to the next connected object; see
    /// [`Finish`].
    pub fn finish(&mut self, finish: Finish) -> &mut Self {
        self.actions.finish = Some(finish);
        self
    }

    /// Changes the response with the [`JsonAction`] builder, for commands
    /// without a shortcut here such as `sheet` or `opendialog`.
    pub fn apply(&mut self, change: impl FnOnce(JsonAction) -> JsonAction) -> &mut Self {
        self.actions = change(std::mem::take(&mut self.actions));
        self
    }

    /// True when the response changes nothing.
    pub fn is_empty(&self) -> bool {
        self.to_json()
            .as_object()
            .is_some_and(serde_json::Map::is_empty)
    }

    /// The response as JSON.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(&self.actions)
            .unwrap_or_else(|_| Value::Object(serde_json::Map::new()))
    }

    /// Problems Alfred would not report: unknown commands it ignores, and
    /// `finish` variables it drops. See [`check_actions`](super::check_actions).
    pub fn problems(&self) -> Vec<Problem> {
        super::check_actions(&self.to_json())
    }

    /// Writes the response as one JSON object.
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<()> {
        serde_json::to_writer(&mut writer, &self.to_json())?;
        writeln!(writer)?;
        Ok(())
    }
}

impl From<JsonAction> for UiResponse {
    fn from(actions: JsonAction) -> Self {
        UiResponse { actions }
    }
}

/// The error notification sent when a script fails: Alfred would drop the
/// output of a failed script, so the user would otherwise see nothing.
fn failure_response(message: &str, ctx: &UiContext) -> UiResponse {
    let title = ctx
        .var("alfred_workflow_name")
        .filter(|name| !name.is_empty())
        .unwrap_or("Workflow error");
    let mut response = UiResponse::new();
    response.notify(title, message);
    response
}

fn panic_response(payload: &(dyn std::any::Any + Send), ctx: &UiContext) -> UiResponse {
    let detail = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_string());
    failure_response(&format!("Internal error: {detail}"), ctx)
}

/// A UI script backend, the Alfred UI counterpart of [`Runnable`](crate::Runnable).
///
/// ```
/// use alfrusco::ui::{UiContext, UiResponse, UiRunnable};
///
/// struct Rename;
///
/// impl UiRunnable for Rename {
///     type Error = alfrusco::Error;
///
///     fn run(self, ctx: &UiContext, response: &mut UiResponse) -> Result<(), Self::Error> {
///         let title = ctx.resolved_arg(0).unwrap_or("Untitled");
///         response.update("title", "text", title);
///         Ok(())
///     }
/// }
///
/// let response = UiContext::new().with_arg("Groceries").run(Rename);
/// assert_eq!(response.to_json()["update"]["ids"]["title"]["text"], "Groceries");
/// ```
pub trait UiRunnable {
    /// The error `run` can return.
    type Error: fmt::Display;

    /// Builds the response for one script invocation.
    fn run(
        self,
        ctx: &UiContext,
        response: &mut UiResponse,
    ) -> std::result::Result<(), Self::Error>;

    /// The response written when `run` fails. Anything `run` added before it
    /// failed is discarded. The default posts a notification with the error,
    /// titled with the workflow name.
    fn error_response(error: &Self::Error, ctx: &UiContext) -> UiResponse {
        failure_response(&error.to_string(), ctx)
    }
}

/// The async form of [`UiRunnable`].
// async_trait marks the boxed future `#[must_use]`, which clippy flags as
// redundant on Rust 1.99+.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait AsyncUiRunnable {
    /// The error `run_async` can return.
    type Error: fmt::Display;

    /// Builds the response for one script invocation.
    async fn run_async(
        self,
        ctx: &UiContext,
        response: &mut UiResponse,
    ) -> std::result::Result<(), Self::Error>;

    /// The response written when `run_async` fails. The default posts a
    /// notification with the error.
    fn error_response(error: &Self::Error, ctx: &UiContext) -> UiResponse {
        failure_response(&error.to_string(), ctx)
    }
}

/// Runs a UI script backend for this process and writes its JSON actions.
///
/// Call it from `main` and return normally: a non-zero exit would make Alfred
/// drop the output. Errors and panics become an error notification, the error
/// is also written to stderr (which Alfred logs), and a response with nothing
/// to do writes `{}`, because Alfred treats empty output as an error.
///
/// ```no_run
/// use alfrusco::ui::{execute_ui, UiContext, UiResponse, UiRunnable};
///
/// struct Tick;
/// impl UiRunnable for Tick {
///     type Error = alfrusco::Error;
///     fn run(self, _ctx: &UiContext, response: &mut UiResponse) -> Result<(), Self::Error> {
///         response.update("clock", "text", "12:00");
///         Ok(())
///     }
/// }
///
/// fn main() {
///     execute_ui(&UiContext::from_env(), Tick, &mut std::io::stdout());
/// }
/// ```
pub fn execute_ui<R: UiRunnable>(ctx: &UiContext, runnable: R, writer: &mut dyn io::Write) {
    write_response(&ctx.run(runnable), writer);
}

/// The async form of [`execute_ui`].
pub async fn execute_ui_async<R: AsyncUiRunnable>(
    ctx: &UiContext,
    runnable: R,
    writer: &mut dyn io::Write,
) {
    write_response(&ctx.run_async(runnable).await, writer);
}

fn write_response(response: &UiResponse, writer: &mut dyn io::Write) {
    for problem in response.problems() {
        eprintln!("alfrusco: {problem}");
    }
    if let Err(error) = response.write(writer) {
        eprintln!("alfrusco: could not write the UI response: {error}");
    }
}

/// Polls a future inside `catch_unwind`, so a panic in an async backend
/// becomes an error response like a panic in a sync one.
struct CatchUnwind<F> {
    future: Pin<Box<F>>,
}

impl<F: Future> Future for CatchUnwind<F> {
    type Output = std::result::Result<F::Output, Box<dyn std::any::Any + Send>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = self.future.as_mut();
        match panic::catch_unwind(AssertUnwindSafe(|| future.poll(cx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Err(payload) => Poll::Ready(Err(payload)),
        }
    }
}
