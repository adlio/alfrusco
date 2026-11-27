//! Golden tests for `alfrusco::ui`: each test builds a realistic Alfred UI
//! and compares the serialized `ui.json` with a fixture in
//! `tests/fixtures/ui/`.

use alfrusco::ui::*;
use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/ui/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn assert_golden(ui: &Ui, name: &str) {
    let mut bytes = Vec::new();
    ui.write_pretty(&mut bytes).unwrap();
    let actual: Value = serde_json::from_slice(&bytes).unwrap();
    let expected = fixture(name);
    assert!(
        actual == expected,
        "{name}.json differs.\n--- expected\n{}\n--- actual\n{}",
        serde_json::to_string_pretty(&expected).unwrap(),
        serde_json::to_string_pretty(&actual).unwrap(),
    );
}

/// A grid of buttons that each play a system sound, built from one template.
#[test]
fn sound_board() {
    let sounds = [
        "Basso", "Blow", "Bottle", "Frog", "Funk", "Glass", "Hero", "Morse", "Ping", "Pop", "Purr",
        "Tink",
    ];
    let sound_button = Button::new(Placeholder::param("sound"))
        .on(
            Trigger::Actioned,
            JsonAction::new().sound(Placeholder::param("sound")),
        )
        .menu_item(
            MenuItem::new("Say the name")
                .action(JsonAction::new().speak(Placeholder::param("sound"))),
        );

    let grid = View::grid(3, 4)
        .spacing(6)
        .region(Region::Centre)
        .children(sounds.map(|sound| Template::new("soundbutton").param("sound", sound)));
    let root = View::border()
        .spacing(8)
        .view_shortcut(Shortcut::new("⌘1", JsonAction::new().sound("Basso")))
        .child(
            Label::new("Click a sound")
                .font(
                    Font::new()
                        .size(FontSize::Large)
                        .weight(FontWeight::SemiBold),
                )
                .text_align(TextAlign::Centre)
                .region(Region::Top),
        )
        .child(grid);

    let ui = Ui::new(
        Window::new(root)
            .title("Sound Board")
            .size(Size::with_height(320))
            .padding(10),
    )
    .bundle_id("com.example.soundboard")
    .template("soundbutton", sound_button);

    assert_golden(&ui, "sound_board");
}

/// A search field that filters a grid of colour swatches and passes the
/// chosen hex value on to the next workflow object.
#[test]
fn colour_search() {
    let swatches = [
        ("teal", "Teal", "#008080"),
        ("coral", "Coral", "#ff7f50"),
        ("mustard", "Mustard", "#e1ad01"),
        ("plum", "Plum", "#8e4585"),
    ];
    let rows = swatches.map(|(uid, name, hex)| {
        Row::new()
            .uid(uid)
            .value("swatch", "background", hex)
            .value("name", "text", name)
    });

    let cell = Cell::new(
        View::stack().id("tile").corner_radius(6).child(
            View::border()
                .padding(4)
                .spacing(6)
                .child(
                    View::stack()
                        .id("swatch")
                        .corner_radius(4)
                        .region(Region::Centre),
                )
                .child(
                    Label::empty()
                        .id("name")
                        .text_align(TextAlign::Centre)
                        .region(Region::Bottom),
                ),
        ),
    )
    .height(96)
    .init("name", "colour", Colour::named("labelColor"))
    .init("tile", "background", Colour::none())
    .selected("name", "colour", Colour::alfred_selected_result_text())
    .selected(
        "tile",
        "background",
        Colour::alfred_selected_result_background(),
    );

    let swatch_grid = DataView::new(
        cell,
        Data::rows(rows).filter(
            Filter::new("name.text", Placeholder::value("query", "text"))
                .match_mode(MatchMode::WordMatchAnyOrder),
        ),
    )
    .id("swatches")
    .cols(3)
    .spacing(4)
    .placeholder("No colours match")
    .behaviour(DataViewBehaviour::new().selection(Selection::new().wrap(SelectionWrap::Vertical)))
    .region(Region::Centre);

    let search = TextField::new()
        .id("query")
        .placeholder("Search colours")
        .font(Font::new().size(FontSize::ExtraLarge))
        .text_insets(TextInsets::new(8, 8))
        .focus_ring(false)
        .focus(true)
        .region(Region::Top)
        .on(
            Trigger::Changed,
            Action::function("swatches", function::FILTER_DATA),
        )
        .on(
            Trigger::Actioned,
            JsonAction::new().finish(
                Finish::new()
                    .args(Placeholder::value("swatches", "selected.swatch.background"))
                    .variable(
                        "colourname",
                        Placeholder::value("swatches", "selected.name.text"),
                    ),
            ),
        );

    let arrows = [
        ("↑", function::SELECT_UP),
        ("↓", function::SELECT_DOWN),
        ("←", function::SELECT_PREVIOUS),
        ("→", function::SELECT_NEXT),
    ];
    let mut root = View::border().spacing(8).background("panel");
    for (key, name) in arrows {
        root = root.view_shortcut(Shortcut::new(key, Action::function("swatches", name)));
    }
    let root = root.child(search).child(swatch_grid);

    let ui = Ui::new(Window::new(root).size(Size::with_height(560)).padding(8)).colour(
        "panel",
        Colour::light_dark(Colour::hex("f4f1ea"), Colour::rgb(30, 30, 34)),
    );

    assert_golden(&ui, "colour_search");
}

/// A form that renders typed text as Markdown, with a picker choosing how new
/// text combines with the preview.
#[test]
fn text_to_markdown_form() {
    let modes = [
        ("Replace", "replace"),
        ("Append", "append"),
        ("Prepend", "prepend"),
        ("Replace last", "replacelast"),
    ];
    let mut mode_picker = Picker::new().id("mode").selected("replace");
    for (text, value) in modes {
        mode_picker = mode_picker.item(PickerItem::new(text, value));
    }

    let options = View::form()
        .spacing(6)
        .region(Region::Top)
        .child(Label::new("Mode:").constraints(Constraints::new().position(Position::Right)))
        .child(mode_picker)
        .child(View::new())
        .child(
            Checkbox::new("Clear the input after rendering")
                .id("clear")
                .checked(true),
        );

    let render = Button::new("Render")
        .shortcut("⌘↩")
        .tooltip("Render the text ⌘↩")
        .on(
            Trigger::Actioned,
            JsonAction::new()
                .variable("source", Placeholder::value("input", "text"))
                .variable("mode", Placeholder::value("mode", "selected.value"))
                .update(
                    Update::new()
                        .property("preview", "text", Placeholder::var("source"))
                        .property("preview", "behaviour", json!({ "mode": "{var:mode}" }))
                        .property("input", "text", ""),
                ),
        );

    let editors = View::grid(1, 2)
        .spacing(8)
        .region(Region::Centre)
        .child(
            View::border()
                .spacing(4)
                .child(Label::new("Text").region(Region::Top))
                .child(
                    TextArea::new()
                        .id("input")
                        .focus(true)
                        .disable_all_substitutions(true)
                        .constraints(
                            Constraints::new()
                                .region(Region::Centre)
                                .size(Size::new(280, 240)),
                        ),
                ),
        )
        .child(
            View::border()
                .spacing(4)
                .child(Label::new("Markdown").region(Region::Top))
                .child(
                    MarkdownArea::new()
                        .id("preview")
                        .behaviour(TextBehaviour::new().scroll(TextScroll::Auto))
                        .region(Region::Centre),
                ),
        );

    let root = View::border()
        .spacing(8)
        .child(options)
        .child(editors)
        .child(
            View::horizontal()
                .spacing(8)
                .region(Region::Bottom)
                .child(render)
                .child(Template::new("components/copybutton.json").param("target", "preview")),
        );

    let ui = Ui::new(Window::new(root).title("Text to Markdown").padding(12));

    assert_golden(&ui, "text_to_markdown");
}

/// A small browser: an address bar, back, forward and reload buttons, and a
/// web view whose URL comes from the launch argument.
#[test]
fn web_browser() {
    let nav_button = |id: &str, symbol: &str, name: &str, tip: &str| {
        Button::with_image(Image::symbol(symbol))
            .id(id)
            .bordered(false)
            .tooltip(tip)
            .on(Trigger::Actioned, Action::function("page", name))
    };

    let toolbar = View::border()
        .spacing(6)
        .region(Region::Top)
        .child(
            View::horizontal()
                .spacing(2)
                .region(Region::Left)
                .child(nav_button("back", "chevron.left", function::BACK, "Back"))
                .child(nav_button(
                    "forward",
                    "chevron.right",
                    function::FORWARD,
                    "Forward",
                )),
        )
        .child(
            TextField::new()
                .id("address")
                .text_insets(TextInsets::default().horizontal(4))
                .region(Region::Centre)
                .on(
                    Trigger::Actioned,
                    JsonAction::new()
                        .variable("url", Placeholder::value_of_self("text"))
                        .update(Update::new().property("page", "src", Placeholder::var("url"))),
                ),
        )
        .child(
            nav_button("reload", "arrow.clockwise", function::RELOAD, "Reload ⌘R")
                .shortcut("⌘R")
                .region(Region::Right),
        );

    let page = WebKitView::url(Placeholder::var("url"))
        .id("page")
        .open_links_externally_matching("^mailto:")
        .region(Region::Centre)
        .on(
            Trigger::Changed,
            JsonAction::new()
                .variable("url", Placeholder::value_of_self("src"))
                .update(Update::new().property("address", "text", Placeholder::var("url"))),
        );

    let root = View::border()
        .padding(8)
        .spacing(8)
        .view_shortcut(Shortcut::new(
            "⌘L",
            JsonAction::new().update(Update::new().property("address", "focus", true)),
        ))
        .child(toolbar)
        .child(page);

    let ui = Ui::new(
        Window::new(root)
            .title("Browser")
            .title_visibility(false)
            .titlebar_appears_transparent(true)
            .full_size_content_view(true)
            .toolbar(Toolbar::unified())
            .size(Size::new(960, 720))
            .min_size(Size::new(480, 320))
            .frame_autosave(true),
    )
    .bundle_id("com.example.browser")
    .variable_config(
        "url",
        VariableConfig::new()
            .src(VariableSource::Arg(0))
            .default_value("https://example.com"),
    )
    .debug(DebugSettings::new().reload_shortcut("⌘⇧R"));

    assert_golden(&ui, "web_browser");
}
