//! Serialization checks for every `alfrusco::ui` builder, against the
//! property names in the Alfred UI documentation.

use alfrusco::ui::*;
use alfrusco::IconResize;
use serde_json::{json, Value};

fn to_json(value: impl serde::Serialize) -> Value {
    serde_json::to_value(value).unwrap()
}

#[test]
fn colours_cover_every_format() {
    assert_eq!(to_json(Colour::hex("#0000ff44")), json!("#0000ff44"));
    assert_eq!(to_json(Colour::hex("0000ff")), json!("#0000ff"));
    assert_eq!(to_json(Colour::rgb(0, 0, 255)), json!("rgb(0, 0, 255)"));
    assert_eq!(
        to_json(Colour::rgba(0, 0, 255, 0.3)),
        json!("rgb(0, 0, 255, 0.3)")
    );
    assert_eq!(
        to_json(Colour::hsv(280, 50, 100)),
        json!("hsv(280, 50, 100)")
    );
    assert_eq!(
        to_json(Colour::hsva(280, 50, 100, 0.5)),
        json!("hsv(280, 50, 100, 0.5)")
    );
    assert_eq!(
        to_json(Colour::named("systemOrangeColor")),
        json!("systemOrangeColor")
    );
    assert_eq!(
        to_json(Colour::from(String::from("Banana"))),
        json!("Banana")
    );
    assert_eq!(
        to_json(Colour::from(Placeholder::var("accent"))),
        json!("{var:accent}")
    );
    assert_eq!(to_json(Colour::none()), Value::Null);

    let adaptive = Colour::light_dark("#ffffff", Colour::named("systemGrayColor"));
    let expected = json!({ "light": "#ffffff", "dark": "systemGrayColor" });
    assert_eq!(to_json(&adaptive), expected);
    assert_eq!(Value::from(adaptive), expected);
    assert_eq!(Value::from(Colour::none()), Value::Null);
    assert_eq!(Value::from(Colour::new("red")), json!("red"));
}

#[test]
fn fonts_images_and_sizes() {
    let sizes = [
        (FontSize::Pixels(24), json!(24)),
        (FontSize::Default, json!("default")),
        (FontSize::Hairline, json!("hairline")),
        (FontSize::ExtraSmall, json!("extrasmall")),
        (FontSize::Small, json!("small")),
        (FontSize::Medium, json!("medium")),
        (FontSize::Large, json!("large")),
        (FontSize::ExtraLarge, json!("extralarge")),
        (FontSize::Ginormous, json!("ginormous")),
    ];
    for (size, expected) in sizes {
        assert_eq!(to_json(size), expected);
    }

    assert_eq!(
        to_json(
            Font::new()
                .pixels(13)
                .monospaced(true)
                .weight(FontWeight::UltraLight)
                .width(FontWidth::Condensed)
                .set("design", "rounded")
        ),
        json!({
            "size": 13,
            "monospaced": true,
            "weight": "ultralight",
            "width": "condensed",
            "design": "rounded"
        })
    );
    assert_eq!(
        to_json(Font::new().monospaced_digits()),
        json!({ "monospaced": "digits" })
    );

    assert_eq!(
        to_json(
            Image::file("a.png")
                .size(Size::new(16, 16))
                .set("template", true)
        ),
        json!({ "type": "file", "src": "a.png", "size": { "width": 16, "height": 16 }, "template": true })
    );
    assert_eq!(
        to_json(Image::file_icon("~/Desktop")),
        json!({ "type": "fileicon", "src": "~/Desktop" })
    );
    assert_eq!(
        to_json(Image::file_type_icon("notes.md")),
        json!({ "type": "filetypeicon", "src": "notes.md" })
    );
    assert_eq!(
        to_json(Size::with_height(10).width(20).set("depth", 1)),
        json!({ "width": 20, "height": 10, "depth": 1 })
    );
    assert_eq!(
        to_json(TextInsets::new(1, 2).set("top", 3)),
        json!({ "horizontal": 1, "vertical": 2, "top": 3 })
    );
}

#[test]
fn menus_shortcuts_and_accessibility() {
    let menu = MenuItem::new("Sort")
        .state(MenuItemState::Mixed)
        .submenu_item(MenuItem::new("By name").state(MenuItemState::On))
        .submenu_item(MenuItem::separator())
        .submenu_item(
            MenuItem::new("By date")
                .state(MenuItemState::Off)
                .action(Action::script("sort.sh")),
        )
        .set("key", "s");
    assert_eq!(
        to_json(menu),
        json!({
            "text": "Sort",
            "state": "mixed",
            "submenu": [
                { "text": "By name", "state": "on" },
                { "type": "separator" },
                { "text": "By date", "state": "off", "action": { "type": "script", "src": "sort.sh" } }
            ],
            "key": "s"
        })
    );

    let shortcut = Shortcut::new("⌘J", Action::log("first"))
        .action(Action::log("second"))
        .set("repeat", false);
    assert_eq!(
        to_json(shortcut),
        json!({
            "shortcut": "⌘J",
            "action": [
                { "type": "log", "args": "first" },
                { "type": "log", "args": "second" }
            ],
            "repeat": false
        })
    );

    assert_eq!(
        to_json(
            Accessibility::label("Save")
                .help("Saves the note.")
                .set("role", "button")
        ),
        json!({ "label": "Save", "help": "Saves the note.", "role": "button" })
    );
}

#[test]
fn shared_component_properties() {
    let label = Label::new("Status")
        .id("status")
        .constraints(Constraints::new().position(Position::TopLeft))
        .region(Region::Left)
        .event(
            Event::new(Trigger::Clicked)
                .trigger(Trigger::DoubleClicked)
                .action(Action::log("hit")),
        )
        .focus(false)
        .hidden(true)
        .opacity(0.5)
        .enabled(false)
        .tooltip("Current status")
        .menu_item(MenuItem::new("Copy").action(JsonAction::new().copy(ClipboardCopy::text("ok"))))
        .shortcut("⌘S")
        .accessibility(Accessibility::label("Status"))
        .set("vibrancy", true);
    assert_eq!(
        to_json(Component::from(label)),
        json!({
            "type": "label",
            "id": "status",
            "constraints": { "position": "topleft", "region": "left" },
            "events": [
                { "triggers": ["clicked", "doubleclicked"], "action": { "type": "log", "args": "hit" } }
            ],
            "focus": false,
            "hidden": true,
            "opacity": 0.5,
            "enabled": false,
            "tooltip": "Current status",
            "menu": [
                {
                    "text": "Copy",
                    "action": { "type": "json", "src": { "copy": { "type": "text", "src": "ok" } } }
                }
            ],
            "shortcut": "⌘S",
            "accessibility": { "label": "Status" },
            "text": "Status",
            "vibrancy": true
        })
    );
}

#[test]
fn constraints_and_layouts() {
    let constraints = Constraints::new()
        .stretch(Stretch::Horizontal)
        .grid(
            GridCell::at(1, 2)
                .col(3)
                .row(4)
                .colspan(2)
                .rowspan(3)
                .colweight(0.5)
                .rowweight(1.0)
                .set("align", "start"),
        )
        .insets(Insets::new(1, 2, 3, 4).set("unit", "px"))
        .square(48)
        .set("priority", 2);
    assert_eq!(
        to_json(constraints),
        json!({
            "stretch": "horizontal",
            "grid": {
                "col": 3, "row": 4, "colspan": 2, "rowspan": 3,
                "colweight": 0.5, "rowweight": 1.0, "align": "start"
            },
            "insets": { "top": 1, "right": 2, "bottom": 3, "left": 4, "unit": "px" },
            "size": 48,
            "priority": 2
        })
    );
    assert_eq!(
        to_json(Insets::all(8).top(0)),
        json!({ "top": 0, "right": 8, "bottom": 8, "left": 8 })
    );
    assert_eq!(
        to_json(Insets::default().left(2).right(3).bottom(4)),
        json!({ "right": 3, "bottom": 4, "left": 2 })
    );

    let layouts = [
        (View::stack(), json!({ "type": "view", "layout": "stack" })),
        (
            View::vertical(),
            json!({ "type": "view", "layout": "inline", "direction": "vertical" }),
        ),
        (
            View::inline(Direction::Horizontal),
            json!({ "type": "view", "layout": "inline", "direction": "horizontal" }),
        ),
        (View::form(), json!({ "type": "view", "layout": "form" })),
        (
            View::flex(3, 2),
            json!({ "type": "view", "layout": "flex", "rows": 3, "cols": 2 }),
        ),
        (
            View::with_layout(Layout::Grid).rows(2).cols(5),
            json!({ "type": "view", "layout": "grid", "rows": 2, "cols": 5 }),
        ),
        (
            View::new()
                .layout(Layout::Border)
                .direction(Direction::Vertical),
            json!({ "type": "view", "layout": "border", "direction": "vertical" }),
        ),
    ];
    for (view, expected) in layouts {
        assert_eq!(to_json(Component::from(view)), expected);
    }

    let flex_child = View::new()
        .constraints(Constraints::new().grid(GridCell::default().colspan(2)))
        .background("White");
    assert_eq!(
        to_json(Component::from(flex_child)),
        json!({ "type": "view", "constraints": { "grid": { "colspan": 2 } }, "background": "White" })
    );
}

#[test]
fn view_properties() {
    let view = View::new()
        .clip_to_bounds(true)
        .corner_radius(12)
        .appearance(Appearance::Dark)
        .outline(Border::new(1, Colour::named("separatorColor")).set("dash", 2))
        .focusable(true)
        .veto_movable_by_background(true)
        .set("material", "sidebar");
    assert_eq!(
        to_json(Component::from(view)),
        json!({
            "type": "view",
            "cliptobounds": true,
            "cornerradius": 12,
            "appearance": "dark",
            "border": { "width": 1, "colour": "separatorColor", "dash": 2 },
            "focusable": true,
            "vetomovablebybackground": true,
            "material": "sidebar"
        })
    );
}

#[test]
fn text_components() {
    let label = Label::empty()
        .text("Hi")
        .colour(Colour::named("labelColor"))
        .font(Font::new().size(FontSize::Small))
        .selectable(true)
        .line_break(LineBreak::WordWrap)
        .text_align(TextAlign::Justify);
    assert_eq!(
        to_json(Component::from(label)),
        json!({
            "type": "label", "text": "Hi", "colour": "labelColor", "font": { "size": "small" },
            "selectable": true, "linebreak": "wordwrap", "textalign": "justify"
        })
    );

    let field = TextField::new()
        .text("draft")
        .multiline(true)
        .colour("textColor")
        .placeholder_colour(Colour::none())
        .bordered(false)
        .background(Colour::none())
        .editable(true)
        .selectable(true)
        .line_break(TextFieldLineBreak::Wrap);
    assert_eq!(
        to_json(Component::from(field)),
        json!({
            "type": "textfield", "text": "draft", "multiline": true, "colour": "textColor",
            "placeholdercolour": null, "bordered": false, "background": null,
            "editable": true, "selectable": true, "linebreak": "wrap"
        })
    );

    let password = PasswordField::new()
        .id("token")
        .font(Font::new().monospaced(true))
        .colour("textColor")
        .bordered(true)
        .background("controlBackgroundColor")
        .text_insets(TextInsets::default().vertical(6))
        .focus_ring(true);
    assert_eq!(
        to_json(Component::from(password)),
        json!({
            "type": "passwordfield", "id": "token", "font": { "monospaced": true },
            "colour": "textColor", "bordered": true, "background": "controlBackgroundColor",
            "textinsets": { "vertical": 6 }, "focusring": true
        })
    );

    let area = TextArea::new()
        .text("log")
        .background("textBackgroundColor")
        .font(Font::new().size(FontSize::Medium))
        .colour("textColor")
        .border_type(TextAreaBorder::Solid)
        .editable(false)
        .selectable(true)
        .behaviour(
            TextBehaviour::new()
                .mode(TextMode::Append)
                .scroll(TextScroll::End)
                .set("limit", 100),
        )
        .auto_adjust_content_insets(false)
        .text_insets(TextInsets::new(4, 4));
    assert_eq!(
        to_json(Component::from(area)),
        json!({
            "type": "textarea", "text": "log", "background": "textBackgroundColor",
            "font": { "size": "medium" }, "colour": "textColor", "bordertype": "solid",
            "editable": false, "selectable": true,
            "behaviour": { "mode": "append", "scroll": "end", "limit": 100 },
            "autoadjustcontentinsets": false, "textinsets": { "horizontal": 4, "vertical": 4 }
        })
    );

    let markdown = MarkdownArea::new()
        .text("# Notes")
        .src("notes.md")
        .background(Colour::none())
        .font(Font::new().size(FontSize::Hairline))
        .colour("labelColor")
        .line_spacing(2.0)
        .line_height_multiple(1.2)
        .paragraph_spacing(8.0)
        .list_spacing(4.0)
        .auto_adjust_content_insets(true)
        .uri_schemes(["http", "https", "mailto"])
        .behaviour(TextBehaviour::new().mode(TextMode::ReplaceLast));
    assert_eq!(
        to_json(Component::from(markdown)),
        json!({
            "type": "markdownarea", "text": "# Notes", "src": "notes.md", "background": null,
            "font": { "size": "hairline" }, "colour": "labelColor", "linespacing": 2.0,
            "lineheightmultiple": 1.2, "paragraphspacing": 8.0, "listspacing": 4.0,
            "autoadjustcontentinsets": true, "urischemes": ["http", "https", "mailto"],
            "behaviour": { "mode": "replacelast" }
        })
    );
}

#[test]
fn controls() {
    let button = Button::new("Go")
        .font(Font::new().weight(FontWeight::Heavy))
        .border_shape(BorderShape::RoundedRectangle)
        .effect(ButtonEffect::Glass)
        .content_tint_colour("systemBlueColor");
    assert_eq!(
        to_json(Component::from(button)),
        json!({
            "type": "button", "text": "Go", "font": { "weight": "heavy" },
            "bordershape": "roundedrectangle", "effect": "glass",
            "contenttintcolour": "systemBlueColor"
        })
    );

    let picker = Picker::new()
        .item(PickerItem::new("One", "1").set("tooltip", "first"))
        .item(PickerItem::separator())
        .font(Font::new())
        .bordered(false)
        .border_shape(BorderShape::Capsule);
    assert_eq!(
        to_json(Component::from(picker)),
        json!({
            "type": "picker",
            "items": [{ "text": "One", "value": "1", "tooltip": "first" }, { "type": "separator" }],
            "font": {}, "bordered": false, "bordershape": "capsule"
        })
    );

    let progress = Progress::new()
        .style(ProgressStyle::Round)
        .thickness(4)
        .progress(0.25)
        .colour("systemOrangeColor")
        .background(Colour::none());
    assert_eq!(
        to_json(Component::from(progress)),
        json!({
            "type": "progress", "style": "round", "thickness": 4, "progress": 0.25,
            "colour": "systemOrangeColor", "background": null
        })
    );

    let image = ImageView::empty()
        .image(Image::symbol("star"))
        .resize(IconResize::Centre)
        .background("windowBackgroundColor")
        .content_tint_colour("systemYellowColor")
        .corner_radius(8);
    assert_eq!(
        to_json(Component::from(image)),
        json!({
            "type": "imageview", "image": { "type": "symbol", "src": "star" }, "resize": "centre",
            "background": "windowBackgroundColor", "contenttintcolour": "systemYellowColor",
            "cornerradius": 8
        })
    );
    assert_eq!(
        to_json(Component::from(ImageView::new(Image::file("logo.png")))),
        json!({ "type": "imageview", "image": { "type": "file", "src": "logo.png" } })
    );

    assert_eq!(
        to_json(Component::from(
            WebKitView::html("<b>hi</b>").open_links_externally(true)
        )),
        json!({ "type": "webkitview", "text": "<b>hi</b>", "openlinksexternally": true })
    );
    assert_eq!(
        to_json(Component::from(
            FileComponent::new("components/ok.json")
                .id("ok")
                .region(Region::Bottom)
        )),
        json!({ "type": "file", "id": "ok", "constraints": { "region": "bottom" }, "src": "components/ok.json" })
    );
}

#[test]
fn raw_and_placeholder_components() {
    let view = View::border()
        .child(Placeholder::file("components/search.json"))
        .child(Component::placeholder("{file:components/list.json}"))
        .child(Component::raw(json!({ "type": "slider", "value": 3 })));
    assert_eq!(
        to_json(Component::from(view)),
        json!({
            "type": "view",
            "layout": "border",
            "components": [
                "{file:components/search.json}",
                "{file:components/list.json}",
                { "type": "slider", "value": 3 }
            ]
        })
    );
}

#[test]
fn dataview_edits_and_settings() {
    let cell = Cell::new(Label::empty().id("title")).set("reuse", "title");
    let view = DataView::new(
        cell,
        Data::source(Placeholder::file_raw("data/rows.json")).set("cache", true),
    )
    .padding(2)
    .auto_adjust_content_insets(false)
    .behaviour(
        DataViewBehaviour::new()
            .selection(
                Selection::new()
                    .wrap(SelectionWrap::None)
                    .allow_empty(true)
                    .hover(true)
                    .set("multiple", false),
            )
            .set("drag", true),
    )
    .set("rowheight", 20);
    assert_eq!(
        to_json(Component::from(view)),
        json!({
            "type": "dataview",
            "cell": { "root": { "type": "label", "id": "title" }, "reuse": "title" },
            "data": { "type": "basic", "src": "{fileraw:data/rows.json}", "cache": true },
            "padding": 2,
            "autoadjustcontentinsets": false,
            "behaviour": {
                "selection": { "wrap": "none", "allowempty": true, "hover": true, "multiple": false },
                "drag": true
            },
            "rowheight": 20
        })
    );
}

#[test]
fn dataview_row_edits() {
    let modify = Modify::new()
        .insert(
            InsertRows::new([Row::new().uid("new").value("title", "text", "New")])
                .after("first")
                .before("second")
                .position(InsertPosition::After)
                .autoselect(false)
                .set("animate", true),
        )
        .replace(
            ReplaceRows::new()
                .row(
                    "old",
                    Row::new().value("title", "text", "Renamed").set("extra", 1),
                )
                .autoselect(true)
                .set("animate", false),
        )
        .delete(
            DeleteRows::new(["gone"])
                .autoselect(false)
                .set("animate", true),
        )
        .set("batch", 1);
    let data = Data::change().modify(modify).select("new").filter(
        Filter::new(["title.text", "match"], "{value:q.text}")
            .match_empty(false)
            .set("fuzzy", true),
    );
    let update = Update::new().data("list", data);
    assert_eq!(
        to_json(update)["ids"]["list"]["data"],
        json!({
            "type": "basic",
            "filter": {
                "type": "basic", "args": ["title.text", "match"], "match": "{value:q.text}",
                "matchempty": false, "fuzzy": true
            },
            "modify": {
                "insert": {
                    "position": "after", "uid": "second", "autoselect": false,
                    "src": [{ "uid": "new", "title.text": "New" }], "animate": true
                },
                "replace": {
                    "autoselect": true,
                    "srcmap": { "old": { "title.text": "Renamed", "extra": 1 } },
                    "animate": false
                },
                "delete": { "autoselect": false, "uids": ["gone"], "animate": true },
                "batch": 1
            },
            "select": "new"
        })
    );

    let other_modes = [
        (
            MatchMode::ExactFromStartOrWhitespace,
            "exactfromstartorwhitespace",
        ),
        (MatchMode::ExactFromStart, "exactfromstart"),
        (MatchMode::WordMatchSequential, "wordmatchsequential"),
    ];
    for (mode, name) in other_modes {
        assert_eq!(to_json(mode), json!(name));
    }
    assert_eq!(
        to_json(InsertRows::new([]).position(InsertPosition::Before)),
        json!({ "position": "before", "src": [] })
    );
}

#[test]
fn events_and_actions() {
    let event = Event::new(Trigger::Actioned)
        .confirm(
            Confirm::new("Delete everything?")
                .icon_path("icons/warning.png")
                .informative_text("This can't be undone.")
                .confirm_button_text("Delete")
                .cancel_button_text("Keep")
                .set("style", "critical"),
        )
        .action(
            Action::script("scripts/delete.sh")
                .arg("--all")
                .arg(Placeholder::data("uid"))
                .arg("--force")
                .variable("count", 3)
                .flatten_array_args(true)
                .asynchronous(true)
                .set("timeout", 5),
        )
        .action(Action::json_file("actions/done.json"))
        .set("debounce", 0.2);
    assert_eq!(
        to_json(event),
        json!({
            "triggers": "actioned",
            "confirm": {
                "message": "Delete everything?",
                "iconpath": "icons/warning.png",
                "informativetext": "This can't be undone.",
                "confirmbuttontext": "Delete",
                "cancelbuttontext": "Keep",
                "style": "critical"
            },
            "action": [
                {
                    "type": "script",
                    "src": "scripts/delete.sh",
                    "args": ["--all", "{data:uid}", "--force"],
                    "variables": { "count": 3 },
                    "flattenarrayargs": true,
                    "async": true,
                    "timeout": 5
                },
                { "type": "json", "src": "actions/done.json" }
            ],
            "debounce": 0.2
        })
    );

    assert_eq!(
        to_json(Action::script("a.sh").args("one").arg("two")),
        json!({ "type": "script", "src": "a.sh", "args": ["one", "two"] })
    );
    assert_eq!(
        to_json(Action::function(
            "notes",
            [function::CARET_END, function::SCROLL_BOTTOM]
        )),
        json!({ "type": "function", "src": "notes", "args": ["caretend", "scrollbottom"] })
    );

    let triggers = [
        (Trigger::Preload, "preload"),
        (Trigger::Launched, "launched"),
        (Trigger::Quit, "quit"),
        (Trigger::TypingStopped, "typingstopped"),
        (Trigger::MouseDown, "mousedown"),
        (Trigger::MouseUp, "mouseup"),
    ];
    for (trigger, name) in triggers {
        assert_eq!(to_json(trigger), json!(name));
    }
    assert_eq!(to_json(ActionType::Log), json!("log"));
}

#[test]
fn json_action_ui_commands() {
    let actions = JsonAction::new()
        .variable("who", "vero")
        .pulse(Pulse::stop())
        .remove(Remove::new(["a", "b"]).relayout(true).set("animate", true))
        .insert(
            Insert::new()
                .component("list", Label::new("one"))
                .component("list", Label::new("two"))
                .set("at", 0),
        )
        .update(
            Update::new()
                .properties(
                    "status",
                    serde_json::Map::from_iter([("text".into(), json!("Done"))]),
                )
                .relayout(true)
                .set("animate", false),
        )
        .replace(
            Replace::new()
                .component("status", TextArea::new().text("Done"))
                .retain_ids(false)
                .set("animate", true),
        )
        .sheet(
            Sheet::show(Label::new("Saved"))
                .id("saved")
                .close_on_escape(false)
                .set("width", 200),
        )
        .set("haptic", "generic");

    assert_eq!(
        to_json(actions),
        json!({
            "variables": { "who": "vero" },
            "pulse": { "interval": 0.0 },
            "remove": { "ids": ["a", "b"], "relayout": true, "animate": true },
            "insert": {
                "ids": { "list": [{ "type": "label", "text": "one" }, { "type": "label", "text": "two" }] },
                "at": 0
            },
            "update": { "relayout": true, "ids": { "status": { "text": "Done" } }, "animate": false },
            "replace": {
                "retainids": false,
                "ids": { "status": { "type": "textarea", "text": "Done" } },
                "animate": true
            },
            "sheet": {
                "id": "saved",
                "closeonescape": false,
                "root": { "type": "label", "text": "Saved" },
                "width": 200
            },
            "haptic": "generic"
        })
    );
}

#[test]
fn json_action_system_commands() {
    let actions = JsonAction::new()
        .share(Share::urls("https://example.com").set("subject", "Look"))
        .sound(Sound::new("Ping").set("volume", 0.5))
        .speak(Speak::new("Done").set("voice", "Samantha"))
        .copy(ClipboardCopy::files(["~/a.txt", "~/b.txt"]).set("transient", true))
        .open(Open::new(SourceType::Url, "https://example.com").set("background", true))
        .open_dialog(
            OpenDialog::new()
                .can_choose_files(false)
                .can_choose_folders(true)
                .file_types("public.folder")
                .folder("~")
                .prompt("Choose")
                .message("Pick a folder")
                .multiselect(false)
                .set_variable("folder")
                .action(JsonAction::new().open(Open::files(Placeholder::var("folder"))))
                .set("hidden", true),
        )
        .window(WindowCommand::resize(Some(800.into()), Some(Dimension::Auto)).set("animate", true))
        .notification(
            Notification::new()
                .post(
                    Post::new("Sync", "Finished")
                        .uid("sync")
                        .set("sound", "Glass"),
                )
                .remove("old")
                .set("group", "sync"),
        )
        .functions(
            Functions::new()
                .call("notes", function::CARET_START)
                .set("delay", 1),
        )
        .popup_menu(
            PopupMenu::new("more")
                .item(MenuItem::new("Help"))
                .set("at", "mouse"),
        )
        .finish(
            Finish::new()
                .args(["a", "b"])
                .flatten_array_args(true)
                .set("close", false),
        );

    assert_eq!(
        to_json(actions),
        json!({
            "share": { "type": "url", "src": "https://example.com", "subject": "Look" },
            "sound": { "src": "Ping", "volume": 0.5 },
            "speak": { "text": "Done", "voice": "Samantha" },
            "copy": { "type": "file", "src": ["~/a.txt", "~/b.txt"], "transient": true },
            "open": { "type": "url", "src": "https://example.com", "background": true },
            "opendialog": {
                "canchoosefiles": false,
                "canchoosefolders": true,
                "filetypes": "public.folder",
                "folder": "~",
                "prompt": "Choose",
                "message": "Pick a folder",
                "multiselect": false,
                "setvariable": "folder",
                "action": { "type": "json", "src": { "open": { "type": "file", "src": "{var:folder}" } } },
                "hidden": true
            },
            "window": { "resize": { "width": 800, "height": "auto" }, "animate": true },
            "notification": {
                "post": { "uid": "sync", "title": "Sync", "text": "Finished", "sound": "Glass" },
                "remove": "old",
                "group": "sync"
            },
            "functions": { "ids": { "notes": "caretstart" }, "delay": 1 },
            "popupmenu": { "src": "more", "menu": [{ "text": "Help" }], "at": "mouse" },
            "finish": { "args": ["a", "b"], "flattenarrayargs": true, "close": false }
        })
    );
}

#[test]
fn json_action_command_variants() {
    assert_eq!(to_json(Sheet::close("saved")), json!({ "close": "saved" }));
    assert_eq!(
        to_json(WindowCommand::resize(None, Some(Dimension::Pixels(300)))),
        json!({ "resize": { "height": 300 } })
    );
    assert_eq!(
        to_json(
            Notification::new()
                .post(Post::new("One", "1"))
                .post(Post::default().set("title", "Two"))
        ),
        json!({ "post": [{ "title": "One", "text": "1" }, { "title": "Two" }] })
    );
    assert_eq!(
        to_json(JsonAction::new().sound("Glass").speak("Hi")),
        json!({ "sound": { "src": "Glass" }, "speak": { "text": "Hi" } })
    );
}

#[test]
fn window_settings() {
    let window = Window::new(View::new())
        .id("main")
        .max_size(Size::new(1200, 900))
        .corner_radius(16)
        .blur(false)
        .movable_by_background(true)
        .level(3)
        .appearance(Appearance::Automatic)
        .borderless(true)
        .closeable(false)
        .minimisable(false)
        .resizeable(false)
        .fullscreenable(false)
        .toolbar(Toolbar::unified_compact().set("style", "expanded"))
        .set("shadow", false);
    assert_eq!(
        to_json(window),
        json!({
            "id": "main",
            "maxsize": { "width": 1200, "height": 900 },
            "cornerradius": 16,
            "blur": false,
            "movablebybackground": true,
            "level": 3,
            "appearance": "automatic",
            "borderless": true,
            "closeable": false,
            "minimisable": false,
            "resizeable": false,
            "fullscreenable": false,
            "toolbar": { "type": "unifiedcompact", "style": "expanded" },
            "root": { "type": "view" },
            "shadow": false
        })
    );
    assert_eq!(to_json(Appearance::Inherit), json!("inherit"));
    assert_eq!(to_json(Appearance::Light), json!("light"));
    assert_eq!(
        to_json(Toolbar::new(ToolbarType::Unified)),
        json!({ "type": "unified" })
    );
}

#[test]
fn top_level_settings() {
    let ui = Ui::new(Window::new(Label::new("Sync")))
        .variable("count", 0)
        .variable("filters", json!({ "mode": "append" }))
        .variable_config(
            "token",
            VariableConfig::new()
                .persist(true)
                .script_env(true)
                .src(VariableSource::Variable)
                .set("secret", true),
        )
        .variable_config("all", VariableConfig::new().src(VariableSource::Args))
        .icon_path("icon.png")
        .log(
            Log::new(LogLevel::Debug)
                .path("sync.log")
                .timestamp(true)
                .colourful(true)
                .console(true)
                .internal(true)
                .set("rotate", true),
        )
        .debug(DebugSettings::new().colourful(true).set("outline", true))
        .pulse(
            Pulse::new("scripts/pulse.sh")
                .args(Placeholder::var("count"))
                .now(true)
                .interval_mode(IntervalMode::Regular)
                .interval(1.5)
                .set("jitter", 0.1),
        )
        .set("minimumversion", "6.0");

    let mut compact = Vec::new();
    ui.write(&mut compact).unwrap();
    let json: Value = serde_json::from_slice(&compact).unwrap();
    assert_eq!(
        json,
        json!({
            "variablesconfig": {
                "all": { "src": "args" },
                "token": { "persist": true, "scriptenv": true, "src": "variable", "secret": true }
            },
            "variables": { "count": 0, "filters": { "mode": "append" } },
            "iconpath": "icon.png",
            "log": {
                "path": "sync.log", "level": "debug", "timestamp": true,
                "colourful": true, "console": true, "internal": true, "rotate": true
            },
            "debug": { "colourful": true, "outline": true },
            "pulse": {
                "src": "scripts/pulse.sh", "args": "{var:count}", "now": true,
                "intervalmode": "regular", "interval": 1.5, "jitter": 0.1
            },
            "window": { "root": { "type": "label", "text": "Sync" } },
            "minimumversion": "6.0"
        })
    );

    let levels = [
        (LogLevel::None, "none"),
        (LogLevel::Error, "error"),
        (LogLevel::Warning, "warning"),
        (LogLevel::Info, "info"),
        (LogLevel::Verbose, "verbose"),
        (LogLevel::All, "all"),
    ];
    for (level, name) in levels {
        assert_eq!(to_json(level), json!(name));
    }
    assert_eq!(to_json(IntervalMode::Delay), json!("delay"));
    assert_eq!(
        to_json(Pulse::change().interval(5.0)),
        json!({ "interval": 5.0 })
    );
}

#[test]
fn placeholders() {
    assert_eq!(Placeholder::var("name").as_str(), "{var:name}");
    assert_eq!(Placeholder::constant("api").to_string(), "{const:api}");
    assert_eq!(Placeholder::file("a.json").to_string(), "{file:a.json}");
    assert_eq!(
        Placeholder::file_raw("b.json").to_string(),
        "{fileraw:b.json}"
    );
    assert_eq!(Placeholder::param("text").to_string(), "{param:text}");
    assert_eq!(
        Placeholder::value("list", "selected.value").to_string(),
        "{value:list.selected.value}"
    );
    assert_eq!(
        Placeholder::value_of_self("checked").to_string(),
        "{value:self.checked}"
    );
    assert_eq!(Placeholder::data("uid").to_string(), "{data:uid}");
    assert_eq!(String::from(Placeholder::var("x")), "{var:x}");
    assert_eq!(Value::from(Placeholder::var("x")), json!("{var:x}"));
    assert_eq!(to_json(Placeholder::var("x")), json!("{var:x}"));
    assert_eq!(
        to_json(Action::log(Placeholder::value("field", "text"))),
        json!({ "type": "log", "args": "{value:field.text}" })
    );
}

#[test]
fn typed_args_and_defaults() {
    let action = Action::script("scripts/run.sh")
        .arg("save")
        .arg(true)
        .arg(3)
        .arg(3.5)
        .arg(Value::Null)
        .arg(json!({ "o": 1 }))
        .arg(json!([1, [2, 3]]))
        .arg(Placeholder::var("ids"))
        .flatten_array_args(true);
    assert_eq!(
        to_json(action),
        json!({
            "type": "script",
            "src": "scripts/run.sh",
            "args": ["save", true, 3, 3.5, null, { "o": 1 }, [1, [2, 3]], "{var:ids}"],
            "flattenarrayargs": true
        })
    );
    assert_eq!(
        to_json(Action::script("a.sh").args(["x", "y"]).arg(1)),
        json!({ "type": "script", "src": "a.sh", "args": ["x", "y", 1] })
    );
    assert_eq!(
        to_json(Finish::new().arg("task").arg(42).args("only")),
        json!({ "args": "only" })
    );
    assert_eq!(
        to_json(Finish::new().args("first").arg(2)),
        json!({ "args": ["first", 2] })
    );

    let defaults = [
        (
            VariableConfig::new().default_value(0),
            json!({ "default": 0 }),
        ),
        (
            VariableConfig::new().default_value(true),
            json!({ "default": true }),
        ),
        (
            VariableConfig::new().default_value(json!({ "k": [1, 2] })),
            json!({ "default": { "k": [1, 2] } }),
        ),
        (
            VariableConfig::new().default_value("light"),
            json!({ "default": "light" }),
        ),
    ];
    for (config, expected) in defaults {
        assert_eq!(to_json(config), expected);
    }
}

#[test]
fn set_replaces_the_typed_value() {
    let ui = Ui::new(Window::new(
        View::stack()
            .child(Label::new("typed").set("text", "from set"))
            .child(DataView::new(
                Cell::new(Label::empty().id("name")),
                Data::rows([])
                    .set("type", "sqlite")
                    .filter(Filter::new("name.text", "{value:q.text}").set("type", "fuzzy")),
            ))
            .child(Button::new("Go").set("type", "glassbutton")),
    ))
    .bundle_id("com.example.typed")
    .set("bundleid", "com.example.set");

    let compact = serde_json::to_string(&ui).unwrap();
    assert_eq!(compact.matches("\"bundleid\"").count(), 1);
    assert!(!compact.contains("typed"), "{compact}");
    let json: Value = serde_json::from_str(&compact).unwrap();
    assert_eq!(json["bundleid"], "com.example.set");
    let components = &json["window"]["root"]["components"];
    assert_eq!(components[0]["text"], "from set");
    assert_eq!(components[1]["data"]["type"], "sqlite");
    assert_eq!(components[1]["data"]["filter"]["type"], "fuzzy");
    assert_eq!(components[2]["type"], "glassbutton");

    let actions = JsonAction::new()
        .sound("Glass")
        .set("sound", json!({ "src": "Ping" }));
    let compact = serde_json::to_string(&actions).unwrap();
    assert_eq!(compact, r#"{"sound":{"src":"Ping"}}"#);
}

#[test]
fn sound_speak_and_data_conversions() {
    let name = String::from("Hero");
    assert_eq!(
        to_json(
            JsonAction::new()
                .sound(name.clone())
                .speak(Placeholder::var("words"))
        ),
        json!({ "sound": { "src": "Hero" }, "speak": { "text": "{var:words}" } })
    );
    assert_eq!(
        to_json(JsonAction::new().sound(Placeholder::param("s")).speak(name)),
        json!({ "sound": { "src": "{param:s}" }, "speak": { "text": "Hero" } })
    );
    assert_eq!(
        Value::from(Data::rows([Row::new().uid("a")])),
        json!({ "type": "basic", "src": [{ "uid": "a" }] })
    );
}
