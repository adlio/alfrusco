//! End-to-end checks that the Script Filter JSON keys added for Alfred 5
//! parity and Alfred 6 reach the output written by `execute`.

use alfrusco::config::TestingProvider;
use alfrusco::{execute, Icon, IconResize, Item, ItemType, Runnable, UniversalAction, Workflow};
use serde_json::{json, Value};

struct FileBrowser;

impl Runnable for FileBrowser {
    type Error = alfrusco::Error;

    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        wf.var("mode", "browse");
        wf.strict_mods(true);
        wf.icon_resize(IconResize::Fit);
        wf.preselect("desktop");
        wf.append_item(
            Item::new("Desktop")
                .uid("desktop")
                .arg("~/Desktop")
                .item_type(ItemType::File)
                .icon(Icon::for_file("~/Desktop").resize(IconResize::Fill))
                .universal_action(UniversalAction::new().file("~/Desktop")),
        );
        wf.append_item(Item::new("Alfred").action(["https://www.alfredapp.com", "Alfred"]));
        Ok(())
    }
}

fn run_to_json<R: Runnable>(runnable: R) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let provider = TestingProvider(dir.path().to_path_buf());
    let mut buffer = Vec::new();
    execute(&provider, runnable, &mut buffer);
    serde_json::from_slice(&buffer).unwrap()
}

#[test]
fn response_level_keys_are_written() {
    let json = run_to_json(FileBrowser);
    assert_eq!(json["variables"], json!({"mode": "browse"}));
    assert_eq!(json["strictmods"], true);
    assert_eq!(json["iconresize"], "fit");
    assert_eq!(json["preselect"], "desktop");
}

#[test]
fn item_level_keys_are_written() {
    let json = run_to_json(FileBrowser);
    let desktop = &json["items"][0];
    assert_eq!(desktop["type"], "file");
    assert_eq!(
        desktop["icon"],
        json!({"type": "fileicon", "path": "~/Desktop", "resize": "fill"})
    );
    assert_eq!(desktop["action"], json!({"file": "~/Desktop"}));

    let alfred = &json["items"][1];
    assert_eq!(
        alfred["action"],
        json!(["https://www.alfredapp.com", "Alfred"])
    );
}

struct Plain;

impl Runnable for Plain {
    type Error = alfrusco::Error;

    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        wf.append_item(Item::new("Plain"));
        Ok(())
    }
}

#[test]
fn unset_keys_are_omitted() {
    let json = run_to_json(Plain);
    for key in ["variables", "strictmods", "iconresize", "preselect"] {
        assert!(json.get(key).is_none(), "{key} should be omitted");
    }
    let item = &json["items"][0];
    assert!(item.get("type").is_none());
    assert!(item.get("action").is_none());
}
