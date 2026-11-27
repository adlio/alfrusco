//! `UiPackage` and `check_actions`: the checks Alfred UI doesn't make, and
//! running a UI script the way Alfred 6.0.3 does.

use std::fs;
use std::path::{Path, PathBuf};

use alfrusco::simulator::{ActionResult, ObjectKind, Severity, Simulator, WorkflowGraph};
use alfrusco::ui::*;
use serde_json::{json, Value};

fn task_workflow() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui_task_workflow")
}

fn task_package() -> UiPackage {
    UiPackage::open(task_workflow().join("task.alfredui")).unwrap()
}

fn messages(problems: &[Problem]) -> Vec<String> {
    problems.iter().map(ToString::to_string).collect()
}

#[cfg(unix)]
fn write_script(dir: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn fixture_package_is_clean() {
    let package = task_package();
    let problems = package.problems();
    assert!(problems.is_empty(), "{:#?}", messages(&problems));
    assert_eq!(package.dir(), task_workflow().join("task.alfredui"));
    // The {file:} header was inlined.
    assert_eq!(
        package.json()["window"]["root"]["components"][0]["components"][0]["id"],
        "title"
    );
    let ids = package.ids();
    for id in ["title", "status", "description", "step", "open"] {
        assert!(ids.contains(id), "{id} in {ids:?}");
    }
}

#[test]
fn check_actions_flags_what_alfred_ignores() {
    assert_eq!(check_actions(&json!({})), Vec::<Problem>::new());
    let all_known = json!({
        "variables": {}, "pulse": { "interval": 0 }, "remove": { "ids": "a" },
        "insert": { "ids": {} }, "update": { "ids": {} }, "replace": { "ids": {} },
        "functions": { "ids": {} }, "finish": { "args": "x", "variables": { "n": 1, "s": "t" } },
        "sound": { "src": "Glass" }
    });
    assert!(
        check_actions(&all_known).is_empty(),
        "{:#?}",
        messages(&check_actions(&all_known))
    );

    let array = check_actions(&json!([1, 2]));
    assert_eq!(array[0].severity, Severity::Warning);
    let scalar = check_actions(&json!("hello"));
    assert!(scalar[0].is_error());

    let bad = json!({
        "variables": [],
        "update": { "id": {} },
        "pulse": { "interval": 0.01 },
        "finish": { "variables": { "flag": true, "list": [1] } }
    });
    let found = messages(&check_actions(&bad));
    assert_eq!(found.len(), 5, "{found:#?}");
    assert!(found.contains(&"error: /variables: must be an object".to_string()));
    assert!(found.contains(&"error: /update: needs an `ids` object".to_string()));
    assert!(found
        .iter()
        .any(|m| m.starts_with("warning: /pulse/interval")));
    assert!(found
        .iter()
        .any(|m| m.starts_with("warning: /finish/variables/flag")));
    assert!(found
        .iter()
        .any(|m| m.starts_with("error: /finish/variables/list")));
}

#[test]
fn check_response_against_the_package() {
    let package = task_package();
    let good = json!({
        "update": { "ids": { "status": { "text": "ok" } } },
        "functions": { "ids": { "step": "selectnext" } },
        "insert": { "ids": { "description": [{ "type": "label", "id": "added" }] } },
        "replace": { "ids": { "added": { "type": "label", "text": "x" } } },
        "remove": { "ids": ["added", "status"] },
        "popupmenu": { "src": "open", "menu": [] }
    });
    assert!(
        package.check_response(&good).is_empty(),
        "{:#?}",
        messages(&package.check_response(&good))
    );

    let bad = json!({
        "update": { "ids": { "stauts": { "text": "typo" } } },
        "remove": { "ids": "ghost" },
        "popupmenu": { "src": "nowhere" },
        "spek": { "text": "typo" }
    });
    let found = messages(&package.check_response(&bad));
    assert_eq!(found.len(), 4, "{found:#?}");
    assert!(found.iter().any(|m| m.contains("`stauts`")));
    assert!(found.iter().any(|m| m.contains("`ghost`")));
    assert!(found.iter().any(|m| m.contains("`nowhere`")));
    assert!(found.iter().any(|m| m.contains("`spek`")));
}

#[test]
fn from_ui_checks_a_typed_model() {
    let dir = tempfile::tempdir().unwrap();
    let ui = Ui::new(Window::new(
        View::border()
            .child(TextField::new().id("query").region(Region::Top))
            .child(
                Button::new("Go")
                    .region(Region::Bottom)
                    .on(
                        Trigger::Actioned,
                        Action::function("results", function::SELECT_NEXT),
                    )
                    .on(
                        Trigger::Actioned,
                        JsonAction::new().variable("q", Placeholder::value("qeury", "text")),
                    ),
            ),
    ));
    let found = messages(&UiPackage::from_ui(dir.path(), &ui).problems());
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found
        .iter()
        .any(|m| m.contains("no component has id `results`")));
    assert!(found.iter().any(|m| m.contains("{value:qeury...}")));
}

#[cfg(unix)]
#[test]
fn package_problems_cover_files_scripts_ids_and_types() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("scripts")).unwrap();
    write_script(root, "scripts/ok", "echo '{}'");
    fs::write(root.join("scripts/noexec"), "#!/bin/sh\necho '{}'\n").unwrap();
    fs::write(root.join("not-json.json"), "this is not json").unwrap();
    fs::write(
        root.join("actions.json"),
        r#"{ "update": { "ids": { "ghost": {} } }, "bogus": 1 }"#,
    )
    .unwrap();
    fs::write(
        root.join("loop.json"),
        r#"{ "type": "template", "src": "loop.json" }"#,
    )
    .unwrap();
    fs::write(
        root.join("ui.json"),
        serde_json::to_string(&json!({
            "bundleid": "com.example.broken",
            "extrakey": true,
            "iconpath": "missing-icon.png",
            "pulse": { "src": "scripts/missing", "interval": 1 },
            "templates": {
                "greeting": { "type": "label", "text": "Hello {param:name}", "id": "tpl" }
            },
            "window": {
                "root": {
                    "type": "view",
                    "id": "root",
                    "components": [
                        "{file:missing.json}",
                        "{file:not-json.json}",
                        { "type": "label", "text": "{fileraw:missing-data.json}" },
                        { "type": "slider", "id": "dup" },
                        { "type": "label", "id": "dup" },
                        { "type": "file", "src": "missing-component.json" },
                        { "type": "template", "src": "greeting" },
                        { "type": "template", "src": "greeting", "params": { "name": "x" } },
                        { "type": "template", "src": "nope.json" },
                        { "type": "template" },
                        { "type": "template", "src": "loop.json" },
                        {
                            "type": "button",
                            "events": [
                                { "triggers": "actioned", "action": [
                                    { "type": "script", "src": "scripts/ok", "args": ["{value:dup.text}"] },
                                    { "type": "script", "src": "scripts/noexec" },
                                    { "type": "script", "src": "scripts/missing" },
                                    { "type": "script", "src": "{var:dynamic}" },
                                    { "type": "json", "src": "actions.json" },
                                    { "type": "json", "src": "missing-actions.json" },
                                    { "type": "json", "src": { "update": { "ids": { "phantom": {} } } } },
                                    { "type": "function", "src": "ghostlist", "args": "selectnext" },
                                    { "type": "log", "args": "{value:self.text}" },
                                    { "type": "teleport" },
                                    { "type": "script" }
                                ]}
                            ]
                        }
                    ]
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let package = UiPackage::open(root).unwrap();
    let found = messages(&package.problems());
    let expect = [
        "warning: /extrakey: `extrakey` is not a ui.json key",
        "warning: /iconpath: icon `missing-icon.png` doesn't exist",
        "error: /pulse/src: script `scripts/missing` doesn't exist",
        "error: /window/root/components/0: `{file:missing.json}` doesn't exist",
        "warning: /window/root/components/1: `{file:not-json.json}` isn't a JSON value",
        "error: /window/root/components/2/text: `{fileraw:missing-data.json}` doesn't exist",
        "warning: /window/root/components/3/type: `slider` is not a component type",
        "warning: /window/root/components/4: id `dup` is used more than once",
        "error: /window/root/components/5/src: component file `missing-component.json` doesn't exist",
        "warning: /window/root/components/6: template `greeting` uses `{param:name}`",
        "error: /window/root/components/8/src: template `nope.json` is neither",
        "error: /window/root/components/9: a template needs a `src`",
        "error: /window/root/components/11/events/0/action/1/src: script `scripts/noexec` isn't executable",
        "error: /window/root/components/11/events/0/action/2/src: script `scripts/missing` doesn't exist",
        "error: actions.json:/update/ids/ghost: no component has id `ghost`",
        "warning: actions.json:/bogus: `bogus` is not a JSON action command this version of alfrusco knows",
        "error: /window/root/components/11/events/0/action/5/src: JSON actions file `missing-actions.json`",
        "error: /window/root/components/11/events/0/action/6/src/update/ids/phantom: no component has id `phantom`",
        "error: /window/root/components/11/events/0/action/7/src: no component has id `ghostlist`",
        "error: /window/root/components/11/events/0/action/9/type: `teleport` is not an action type",
        "error: /window/root/components/11/events/0/action/10/src: a `script` action needs a `src`",
    ];
    for want in expect {
        assert!(
            found.iter().any(|m| m.starts_with(want)),
            "missing {want:?} in {found:#?}"
        );
    }
    assert_eq!(found.len(), expect.len(), "{found:#?}");
}

#[test]
fn package_without_a_window_or_ui_json() {
    let dir = tempfile::tempdir().unwrap();
    assert!(UiPackage::open(dir.path()).is_err(), "no ui.json");
    fs::write(dir.path().join("ui.json"), "{ not json").unwrap();
    assert!(UiPackage::open(dir.path()).is_err(), "bad JSON");
    fs::write(dir.path().join("ui.json"), r#"{ "bundleid": "x" }"#).unwrap();
    let found = messages(&UiPackage::open(dir.path()).unwrap().problems());
    assert_eq!(
        found,
        ["error: /window: ui.json needs a window with a root component"]
    );
    fs::write(dir.path().join("ui.json"), "[]").unwrap();
    let found = messages(&UiPackage::open(dir.path()).unwrap().problems());
    assert_eq!(found, ["error: /: ui.json must be an object"]);
}

#[test]
fn file_placeholders_that_nest_forever_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.json"), r#"["{file:a.json}"]"#).unwrap();
    fs::write(
        dir.path().join("ui.json"),
        r#"{ "window": { "root": { "type": "view", "components": "{file:a.json}" } } }"#,
    )
    .unwrap();
    let found = messages(&UiPackage::open(dir.path()).unwrap().problems());
    assert!(
        found.iter().any(|m| m.contains("nests too deep")),
        "{found:#?}"
    );
}

#[cfg(unix)]
#[test]
fn run_script_matches_alfreds_contract() {
    let package = task_package();
    let ctx = UiContext::new()
        .with_args(["whoami", "two words", ""])
        .with_var("task_id", "T-9")
        .with_source("save");
    let run = package.run_script("scripts/backend", &ctx);
    assert!(run.is_ok(), "{run:#?}");
    assert_eq!(run.exit_code, Some(0));
    assert_eq!(
        run.response.as_ref().unwrap()["variables"],
        json!({
            "cwd": "task.alfredui",
            "task_id": "T-9",
            "source": "save",
            "stdin": "eof",
            "leaked": "none",
            "args": "3"
        })
    );

    let pulse = package.run_script(
        "scripts/backend",
        &UiContext::new().as_pulse().with_arg("whoami"),
    );
    assert_eq!(pulse.response.unwrap()["variables"]["source"], "pulse");

    let load = package.run_script(
        "scripts/backend",
        &UiContext::new().with_args(["load", "T-1"]),
    );
    assert!(load.is_ok(), "{load:#?}");
    assert_eq!(
        load.response.unwrap()["update"]["ids"]["description"]["text"],
        "# Task T-1"
    );
}

#[cfg(unix)]
#[test]
fn run_script_reports_output_alfred_would_drop() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("ui.json"),
        r#"{ "window": { "root": { "type": "label", "id": "status" } } }"#,
    )
    .unwrap();
    write_script(root, "empty", "true");
    write_script(root, "text", "echo hello");
    write_script(root, "two", "echo '{}'; echo '{}'");
    write_script(root, "exit3", "echo '{}'; exit 3");
    write_script(
        root,
        "stderr",
        "echo trace >&2; echo '{\"update\":{\"ids\":{\"status\":{}}}}'",
    );
    write_script(
        root,
        "unknown",
        "echo '{\"update\":{\"ids\":{\"nobody\":{}}}}'",
    );
    let package = UiPackage::open(root).unwrap();
    let ctx = UiContext::new();

    for (script, want) in [
        ("empty", "empty output"),
        ("text", "isn't exactly one JSON value"),
        ("two", "isn't exactly one JSON value"),
        ("exit3", "Alfred drops the output of a failed script"),
        ("unknown", "no component has id `nobody`"),
        ("missing", "script launch failed"),
    ] {
        let run = package.run_script(script, &ctx);
        assert!(!run.is_ok(), "{script}");
        assert!(
            messages(&run.problems).iter().any(|m| m.contains(want)),
            "{script}: {:#?}",
            run.problems
        );
    }
    let exited = package.run_script("exit3", &ctx);
    assert_eq!(exited.exit_code, Some(3));

    let traced = package.run_script("stderr", &ctx);
    assert!(traced.is_ok(), "{traced:#?}");
    assert_eq!(traced.stderr, "trace\n");
    assert_eq!(traced.stdout.trim(), r#"{"update":{"ids":{"status":{}}}}"#);
}

#[test]
fn simulator_knows_the_alfred_ui_object() {
    let sim = Simulator::for_workflow_dir(task_workflow()).unwrap();
    let graph = sim.graph();
    let ui = graph.objects().get("UI-TASK").unwrap();
    assert_eq!(ui.kind, ObjectKind::AlfredUi);
    assert_eq!(ui.alfred_ui_path(), Some("task.alfredui"));
    assert_eq!(ui.config_integer("launchmode"), Some(0));
    assert!(!ui.launches_externally());
    assert!(graph.objects()["UI-EXTERNAL"].launches_externally());
    assert_eq!(graph.objects()["SF-TASKS"].alfred_ui_path(), None);
    assert_eq!(graph.alfred_ui_uid("task.alfredui"), Some("UI-TASK"));
    assert_eq!(graph.alfred_ui_uid("other.alfredui"), None);

    // A Script Filter whose results open the UI isn't a dead end...
    let audit = graph.audit_navigation(&["tasks"]);
    assert!(
        audit.iter().all(|d| !d.message.contains("tasks")),
        "{audit:#?}"
    );
    // ...but the external UI's outgoing connection can never run.
    let external: Vec<_> = graph
        .audit_navigation(&[])
        .into_iter()
        .filter(|d| d.object_uid.as_deref() == Some("UI-EXTERNAL"))
        .collect();
    assert_eq!(external.len(), 1);
    assert_eq!(external[0].severity, Severity::Warning);

    let screen = alfrusco::simulator::Screen::from_json(
        r#"{"items":[{"title":"Task","arg":"T-1","valid":true}]}"#,
    )
    .unwrap();
    let _ = screen;
}

#[test]
fn simulator_follows_finish_out_of_the_ui() {
    let sim = Simulator::for_workflow_dir(task_workflow()).unwrap();

    let mut open = UiResponse::new();
    open.finish(Finish::new().arg("T-1").variable("outcome", "open"));
    assert_eq!(
        sim.route_response("UI-TASK", &open),
        ActionResult::OpenedUrl {
            url_template: "https://tasks.example.com/{query}".into()
        }
    );

    let mut other = UiResponse::new();
    other.finish(
        Finish::new()
            .args("T-1")
            .variable("outcome", json!({ "dropped": "by Alfred" })),
    );
    assert_eq!(
        sim.route_response("UI-TASK", &other),
        ActionResult::RanScript {
            target_uid: "RUN-LOG".into()
        }
    );

    let mut external = UiResponse::new();
    external.finish(Finish::new().arg("T-1"));
    assert_eq!(
        sim.route_response("UI-EXTERNAL", &external),
        ActionResult::DeadEnd
    );
    assert_eq!(
        sim.route_response("UI-TASK", &UiResponse::new()),
        ActionResult::DeadEnd
    );
    assert_eq!(sim.route_response("SF-TASKS", &open), ActionResult::DeadEnd);

    let graph = WorkflowGraph::from_plist_file(task_workflow().join("info.plist")).unwrap();
    assert_eq!(
        graph.route_finish(
            "UI-TASK",
            &json!({ "finish": { "args": 7, "variables": { "outcome": "open" } } })
        ),
        ActionResult::OpenedUrl {
            url_template: "https://tasks.example.com/{query}".into()
        }
    );
    // Whole JSON actions, as a script prints them or a ui.json `json` action
    // holds them; no `finish` means no route.
    assert_eq!(
        sim.route_finish("UI-TASK", &json!({ "update": { "ids": {} } })),
        ActionResult::DeadEnd
    );
    let package = task_package();
    let open_button = &package.json()["window"]["root"]["components"][2]["components"][2];
    assert_eq!(open_button["id"], "open");
    assert_eq!(
        sim.route_finish("UI-TASK", &open_button["events"][0]["action"]["src"]),
        ActionResult::OpenedUrl {
            url_template: "https://tasks.example.com/{query}".into()
        }
    );
}

#[test]
fn simulator_routes_results_into_the_ui_and_loads_its_package() {
    let sim = Simulator::for_workflow_dir(task_workflow())
        .unwrap()
        .source_filter("SF-TASKS");
    struct Tasks;
    impl alfrusco::Runnable for Tasks {
        type Error = alfrusco::Error;
        fn run(self, wf: &mut alfrusco::Workflow) -> Result<(), Self::Error> {
            wf.append_item(alfrusco::Item::new("Task one").arg("T-1").valid(true));
            Ok(())
        }
    }
    let screen = sim.run_in_process(Tasks).unwrap();
    assert_eq!(screen.action_first().unwrap().assert_opens_ui(), "UI-TASK");

    let package = sim.ui_package("UI-TASK").unwrap();
    assert_eq!(package.problems(), Vec::<Problem>::new());
    assert!(sim.ui_package("SF-TASKS").is_err());
    assert!(
        sim.ui_package("UI-EXTERNAL").is_err(),
        "external.alfredui isn't in the fixture"
    );

    // Fire the "open" button's event in-process and follow it out of the UI.
    let mut response = UiResponse::new();
    response.finish(
        Finish::new()
            .arg(Placeholder::var("task_id"))
            .variable("outcome", "open"),
    );
    assert_eq!(
        package.check_response(&response.to_json()),
        Vec::<Problem>::new()
    );
    sim.route_response("UI-TASK", &response).assert_opens_url();
    let _: Value = response.to_json();
}

#[test]
#[should_panic(expected = "expected OpenedUi")]
fn assert_opens_ui_panics_on_other_results() {
    ActionResult::DeadEnd.assert_opens_ui();
}

#[test]
fn unknown_commands_are_errors_only_when_they_look_like_typos() {
    let found = check_actions(&json!({ "updat": {}, "notifcation": {}, "haptic": {} }));
    let errors: Vec<&str> = found
        .iter()
        .filter(|p| p.is_error())
        .map(|p| p.location.as_str())
        .collect();
    assert_eq!(errors, ["/notifcation", "/updat"]);
    let warning = found.iter().find(|p| p.location == "/haptic").unwrap();
    assert_eq!(warning.severity, Severity::Warning);
    assert!(
        found[1].message.contains("did you mean `update`?")
            || found[2].message.contains("did you mean `update`?")
    );
}

#[test]
fn pulse_args_follow_the_pulse_rules() {
    let found = messages(&check_actions(&json!({
        "pulse": { "args": ["tick", "{var:n}", "{value:list.selected}", "n-{var:n}"] }
    })));
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].starts_with("error: /pulse/args: pulse arg `{value:list.selected}`"));
    assert!(found[1].starts_with("warning: /pulse/args: pulse arg `n-{var:n}`"));
    assert_eq!(
        check_actions(&json!({ "pulse": { "args": "{var:only}" } })),
        Vec::<Problem>::new()
    );

    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("ui.json"),
        r#"{ "pulse": { "args": "{data:uid}" }, "window": { "root": { "type": "label" } } }"#,
    )
    .unwrap();
    let found = messages(&UiPackage::open(dir.path()).unwrap().problems());
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].starts_with("error: /pulse/args"));
}

/// A package whose `variablesconfig` has one entry of each kind Alfred 6.0.3
/// was observed to treat differently.
#[cfg(unix)]
fn env_rule_package(dir: &Path) -> UiPackage {
    write_script(
        dir,
        "dump",
        r#"printf '{"variables":{"with":"%s","without":"%s","off":"%s","stray":"%s"}}\n' "${with-ABSENT}" "${without-ABSENT}" "${off-ABSENT}" "${stray-ABSENT}""#,
    );
    fs::write(
        dir.join("ui.json"),
        serde_json::to_string(&json!({
            "variablesconfig": {
                "with": { "default": "a", "scriptenv": true },
                "without": { "default": "b" },
                "off": { "default": "c", "scriptenv": false }
            },
            "window": { "root": { "type": "label", "id": "status", "events": [
                { "triggers": "launched", "action": { "type": "script", "src": "dump" } }
            ] } }
        }))
        .unwrap(),
    )
    .unwrap();
    UiPackage::open(dir).unwrap()
}

#[cfg(unix)]
#[test]
fn run_script_passes_only_what_alfred_passes() {
    let dir = tempfile::tempdir().unwrap();
    let package = env_rule_package(dir.path());
    assert_eq!(
        package.script_env_vars().into_iter().collect::<Vec<_>>(),
        ["off", "with"],
        "an entry is passed when its scriptenv key is present, even as false"
    );

    let ctx = UiContext::new()
        .with_var("with", "1")
        .with_var("without", "2")
        .with_var("off", "3")
        .with_var("stray", "4")
        .with_var("alfred_workflow_bundleid", "com.example.env");
    let run = package.run_script("dump", &ctx);
    assert_eq!(
        run.response.as_ref().unwrap()["variables"],
        json!({ "with": "1", "without": "ABSENT", "off": "3", "stray": "ABSENT" })
    );
    let found = messages(&run.problems);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].starts_with("error: env/stray: `stray` isn't a `variablesconfig` entry"));
    assert!(
        found[1].starts_with("error: /variablesconfig/without: `without` has no `scriptenv` key")
    );
    assert!(!run.is_ok());

    let clean = package.run_script("dump", &UiContext::new().with_var("with", "x"));
    assert!(clean.is_ok(), "{clean:#?}");
}

#[cfg(unix)]
#[test]
fn problems_warn_about_entries_scripts_cannot_read() {
    let dir = tempfile::tempdir().unwrap();
    let package = env_rule_package(dir.path());
    let found = messages(&package.problems());
    assert_eq!(
        found,
        ["warning: /variablesconfig/without: `without` has no `scriptenv` key, so Alfred 6.0.3 keeps it out of the scripts' environment; add `\"scriptenv\": true` if a script reads it"]
    );

    // Without any script, a UI-only variable is fine.
    let ui_only = tempfile::tempdir().unwrap();
    fs::write(
        ui_only.path().join("ui.json"),
        r#"{ "variablesconfig": { "theme": { "default": "light" } }, "window": { "root": { "type": "label" } } }"#,
    )
    .unwrap();
    assert_eq!(
        UiPackage::open(ui_only.path()).unwrap().problems(),
        Vec::<Problem>::new()
    );
}

#[test]
fn run_in_process_applies_the_same_rule() {
    struct Echo;
    impl UiRunnable for Echo {
        type Error = alfrusco::Error;
        fn run(self, ctx: &UiContext, response: &mut UiResponse) -> Result<(), Self::Error> {
            let seen = ["task_id", "token", "stray"]
                .map(|name| format!("{name}={}", ctx.var(name).unwrap_or("ABSENT")));
            response.update("status", "text", seen.join(" "));
            Ok(())
        }
    }
    let package = task_package();
    let ctx = UiContext::new()
        .with_var("task_id", "T-3")
        .with_var("stray", "x");
    let run = package.run_in_process(&ctx, Echo);
    assert_eq!(run.exit_code, Some(0));
    assert_eq!(
        run.response.as_ref().unwrap()["update"]["ids"]["status"]["text"],
        "task_id=T-3 token=ABSENT stray=ABSENT"
    );
    assert_eq!(
        run.stdout.trim(),
        run.response.as_ref().unwrap().to_string()
    );
    let found = messages(&run.problems);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].starts_with("error: env/stray"));

    // The plain in-process run passes everything, which is why the package
    // runs exist.
    let raw = ctx.run(Echo).to_json();
    assert_eq!(
        raw["update"]["ids"]["status"]["text"],
        "task_id=T-3 token=ABSENT stray=x"
    );
}
