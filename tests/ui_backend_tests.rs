//! The Alfred UI script backend: what a script receives (`UiContext`) and
//! what it writes (`UiResponse`), checked against the Alfred 6.0.3 contract.

use alfrusco::ui::*;
use serde_json::{json, Value};

/// A backend shaped like a task window's: `load <id>`, `save <field> <value>`.
struct TaskBackend;

impl UiRunnable for TaskBackend {
    type Error = alfrusco::Error;

    fn run(self, ctx: &UiContext, response: &mut UiResponse) -> Result<(), Self::Error> {
        match ctx.arg(0) {
            Some("load") => {
                let id = ctx.resolved_arg(1).unwrap_or("unknown");
                response
                    .variable("token", "tok-1")
                    .update("title", "text", format!("Task {id}"))
                    .rows(
                        "comments",
                        [
                            Row::new().uid("c1").value("body", "text", "First"),
                            Row::new().uid("c2").value("body", "text", "Second"),
                        ],
                    )
                    .call("comments", function::SELECT_FIRST);
            }
            Some("save") => {
                let Some(value) = ctx.resolved_arg(2) else {
                    return Err(alfrusco::Error::Workflow("nothing to save".into()));
                };
                response
                    .update("status", "text", format!("Saved {value}"))
                    .update("save", "enabled", false)
                    .relayout()
                    .notify("Task updated", value);
            }
            Some("tick") if ctx.is_pulse() => {
                response.stop_pulse();
            }
            Some("done") => {
                response.finish(Finish::new().arg("T-1").variable("outcome", "open"));
            }
            Some("panic") => panic!("backend bug"),
            _ => {}
        }
        Ok(())
    }
}

#[test]
fn context_reads_args_and_vars_the_way_alfred_writes_them() {
    let ctx = UiContext::new()
        .with_args(["save", "1", "0", "3.5", "", "{\"o\":1}", "[1,[2,3]]"])
        .with_arg("{value:title.nosuch}")
        .with_var("task_id", "T-42")
        .with_var("flag", "1")
        .with_var("obj", "{\"k\":[1,2]}")
        .with_var("unset", "")
        .with_source("save");

    assert_eq!(ctx.args().len(), 8);
    assert_eq!(ctx.arg(0), Some("save"));
    assert_eq!(ctx.arg(99), None);
    assert_eq!(ctx.arg_bool(1), Some(true));
    assert_eq!(ctx.arg_bool(2), Some(false));
    assert_eq!(ctx.arg_bool(0), None);
    assert_eq!(ctx.arg_json::<f64>(3).unwrap(), 3.5);
    assert_eq!(ctx.resolved_arg(4), None, "null arrives as an empty string");
    assert_eq!(ctx.arg_json::<Value>(5).unwrap(), json!({ "o": 1 }));
    assert_eq!(ctx.arg_json::<Value>(6).unwrap(), json!([1, [2, 3]]));
    assert_eq!(ctx.resolved_arg(7), None, "an unresolved placeholder");
    assert!(ctx.arg_json::<Value>(99).is_err());
    assert!(ctx.arg_json::<Value>(0).is_err());

    assert_eq!(ctx.var("task_id"), Some("T-42"));
    assert_eq!(ctx.var_bool("flag"), Some(true));
    assert_eq!(
        ctx.var_json::<Value>("obj").unwrap(),
        json!({ "k": [1, 2] })
    );
    assert_eq!(ctx.var("unset"), Some(""));
    assert_eq!(ctx.var("missing"), None);
    assert!(ctx.var_json::<Value>("missing").is_err());
    assert_eq!(
        ctx.invocation(),
        Invocation::Event {
            source: Some("save".into())
        }
    );
    assert!(!ctx.is_pulse());

    assert_eq!(
        UiContext::new().invocation(),
        Invocation::Event { source: None },
        "the root view has no id, so alfred_ui_source is empty"
    );
    assert!(UiContext::new().as_pulse().is_pulse());
}

#[test]
fn unresolved_placeholders() {
    for literal in [
        "{value:tf.nosuchprop}",
        "{var:x}",
        "{data:uid}",
        "{const:k}",
        "{param:p}",
    ] {
        assert!(is_unresolved_placeholder(literal), "{literal}");
    }
    for value in ["", "abc", "{value:x", "pre-{var:x}", "{other:x}", "{}"] {
        assert!(!is_unresolved_placeholder(value), "{value}");
    }
}

#[test]
fn context_alfred_variables_and_folders() {
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data/com.example.tasks");
    let cache = home.path().join("cache/com.example.tasks");
    let ctx = UiContext::new()
        .with_var("alfred_workflow_bundleid", "com.example.tasks")
        .with_var("alfred_version", "6.0.3")
        .with_var("alfred_workflow_data", data.to_str().unwrap())
        .with_var("alfred_workflow_cache", cache.to_str().unwrap())
        .with_dir(home.path());

    assert_eq!(ctx.bundle_id(), Some("com.example.tasks"));
    assert_eq!(ctx.alfred_version(), Some("6.0.3"));
    assert_eq!(ctx.ui_dir(), Some(home.path()));
    assert!(
        !data.exists(),
        "Alfred names the folder but doesn't create it"
    );
    assert_eq!(ctx.data_dir().unwrap(), data);
    assert_eq!(ctx.cache_dir().unwrap(), cache);
    assert!(data.is_dir() && cache.is_dir());

    let bare = UiContext::new();
    assert!(bare.data_dir().is_err());
    assert!(bare.cache_dir().is_err());
    assert_eq!(bare.ui_dir(), None);
}

#[test]
fn context_from_env_reads_this_process() {
    let ctx = UiContext::from_env();
    assert!(ctx.ui_dir().is_some());
    assert_eq!(ctx.var("PATH").is_some(), std::env::var("PATH").is_ok());
}

#[test]
fn load_response() {
    let response = UiContext::new().with_args(["load", "T-7"]).run(TaskBackend);
    assert_eq!(
        response.to_json(),
        json!({
            "variables": { "token": "tok-1" },
            "update": {
                "ids": {
                    "title": { "text": "Task T-7" },
                    "comments": {
                        "data": {
                            "type": "basic",
                            "src": [
                                { "uid": "c1", "body.text": "First" },
                                { "uid": "c2", "body.text": "Second" }
                            ]
                        }
                    }
                }
            },
            "functions": { "ids": { "comments": "selectfirst" } }
        })
    );
    assert_eq!(response.problems(), Vec::<Problem>::new());
    assert!(!response.is_empty());
}

#[test]
fn save_response_and_error_response() {
    let saved = UiContext::new()
        .with_args(["save", "title", "Groceries"])
        .run(TaskBackend);
    assert_eq!(
        saved.to_json(),
        json!({
            "update": {
                "relayout": true,
                "ids": {
                    "status": { "text": "Saved Groceries" },
                    "save": { "enabled": false }
                }
            },
            "notification": { "post": { "title": "Task updated", "text": "Groceries" } }
        })
    );

    // A failed run must still write JSON with exit 0, or Alfred shows
    // nothing; the partial response is discarded.
    let failed = UiContext::new()
        .with_args(["save", "title", "{value:title.nosuch}"])
        .with_var("alfred_workflow_name", "Tasks")
        .run(TaskBackend);
    assert_eq!(
        failed.to_json(),
        json!({ "notification": { "post": { "title": "Tasks", "text": "Workflow Error: nothing to save" } } })
    );

    let untitled = UiContext::new()
        .with_args(["save", "title"])
        .run(TaskBackend);
    assert_eq!(
        untitled.to_json()["notification"]["post"]["title"],
        "Workflow error"
    );
}

#[test]
fn panics_become_a_notification() {
    let response = UiContext::new().with_arg("panic").run(TaskBackend);
    assert_eq!(
        response.to_json()["notification"]["post"]["text"],
        "Internal error: backend bug"
    );

    struct OwnedPanic;
    impl UiRunnable for OwnedPanic {
        type Error = alfrusco::Error;
        fn run(self, _: &UiContext, _: &mut UiResponse) -> Result<(), Self::Error> {
            std::panic::panic_any(format!("owned {}", 1))
        }
    }
    let response = UiContext::new().run(OwnedPanic);
    assert_eq!(
        response.to_json()["notification"]["post"]["text"],
        "Internal error: owned 1"
    );

    struct OddPanic;
    impl UiRunnable for OddPanic {
        type Error = alfrusco::Error;
        fn run(self, _: &UiContext, _: &mut UiResponse) -> Result<(), Self::Error> {
            std::panic::panic_any(7_u8)
        }
    }
    let response = UiContext::new().run(OddPanic);
    assert_eq!(
        response.to_json()["notification"]["post"]["text"],
        "Internal error: unknown panic"
    );
}

#[test]
fn custom_error_response() {
    struct Strict;
    impl UiRunnable for Strict {
        type Error = String;
        fn run(self, _: &UiContext, response: &mut UiResponse) -> Result<(), Self::Error> {
            response.update("status", "text", "half done");
            Err("token expired".into())
        }
        fn error_response(error: &String, _ctx: &UiContext) -> UiResponse {
            let mut response = UiResponse::new();
            response.update("status", "text", format!("Reload to edit ({error})"));
            response
        }
    }
    assert_eq!(
        UiContext::new().run(Strict).to_json(),
        json!({ "update": { "ids": { "status": { "text": "Reload to edit (token expired)" } } } })
    );
}

#[test]
fn pulse_and_finish() {
    let tick = UiContext::new()
        .as_pulse()
        .with_arg("tick")
        .run(TaskBackend);
    assert_eq!(tick.to_json(), json!({ "pulse": { "interval": 0.0 } }));

    // Not a pulse: the tick arm doesn't match and nothing changes.
    let event_tick = UiContext::new().with_arg("tick").run(TaskBackend);
    assert!(event_tick.is_empty());

    let done = UiContext::new().with_arg("done").run(TaskBackend);
    assert_eq!(
        done.to_json(),
        json!({ "finish": { "args": ["T-1"], "variables": { "outcome": "open" } } })
    );
}

#[test]
fn empty_response_writes_an_empty_object() {
    let mut out = Vec::new();
    execute_ui(&UiContext::new(), TaskBackend, &mut out);
    assert_eq!(String::from_utf8(out).unwrap(), "{}\n");
    assert!(UiResponse::new().is_empty());
}

#[test]
fn execute_ui_writes_exactly_one_json_value() {
    let mut out = Vec::new();
    execute_ui(
        &UiContext::new().with_args(["save", "title", "x"]),
        TaskBackend,
        &mut out,
    );
    let text = String::from_utf8(out).unwrap();
    assert_eq!(text.lines().count(), 1);
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["update"]["ids"]["status"]["text"], "Saved x");
}

#[test]
fn write_failures_are_reported_not_panicked() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("closed"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    execute_ui(&UiContext::new(), TaskBackend, &mut Broken);
}

#[test]
fn apply_and_problems() {
    let mut response = UiResponse::new();
    response
        .reset_variable("token")
        .pulse(Pulse::change().interval(5000.0))
        .apply(|actions| {
            actions
                .sheet(Sheet::close("confirm"))
                .set("updat", json!({ "ids": {} }))
        })
        .finish(Finish::new().variable("payload", json!({ "x": 1 })));
    let json = response.to_json();
    assert_eq!(json["variables"]["token"], Value::Null);
    assert_eq!(json["sheet"]["close"], "confirm");

    let problems = response.problems();
    let messages: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert_eq!(problems.len(), 3, "{messages:#?}");
    assert!(messages
        .iter()
        .any(|m| m.contains("`updat` is not a JSON action command")));
    assert!(messages.iter().any(|m| m.contains("pulse interval")));
    assert!(messages
        .iter()
        .any(|m| m.contains("/finish/variables/payload") && m.starts_with("error")));

    let from: UiResponse = JsonAction::new().speak("hi").into();
    assert_eq!(from.to_json(), json!({ "speak": { "text": "hi" } }));
}

struct AsyncBackend;

#[async_trait::async_trait]
impl AsyncUiRunnable for AsyncBackend {
    type Error = alfrusco::Error;

    async fn run_async(
        self,
        ctx: &UiContext,
        response: &mut UiResponse,
    ) -> Result<(), Self::Error> {
        tokio::task::yield_now().await;
        match ctx.arg(0) {
            Some("fail") => Err(alfrusco::Error::Workflow("offline".into())),
            Some("panic") => panic!("async bug"),
            _ => {
                response.update("status", "text", "Synced");
                Ok(())
            }
        }
    }
}

#[tokio::test]
async fn async_backend() {
    let mut out = Vec::new();
    execute_ui_async(&UiContext::new(), AsyncBackend, &mut out).await;
    let value: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(value["update"]["ids"]["status"]["text"], "Synced");

    let failed = UiContext::new()
        .with_arg("fail")
        .run_async(AsyncBackend)
        .await;
    assert_eq!(
        failed.to_json()["notification"]["post"]["text"],
        "Workflow Error: offline"
    );

    let panicked = UiContext::new()
        .with_arg("panic")
        .run_async(AsyncBackend)
        .await;
    assert_eq!(
        panicked.to_json()["notification"]["post"]["text"],
        "Internal error: async bug"
    );
}
