//! Typed builders for Alfred UI's `ui.json`.
//!
//! Alfred 6 adds Alfred UI: native macOS windows described in a `ui.json`
//! file inside a `.alfredui` folder and opened by the Alfred UI workflow
//! object. This module models that file so a workflow can build its UI in
//! Rust and write the JSON, instead of editing it by hand.
//!
//! The model only serializes. It doesn't run the UI or answer its events.
//!
//! ```
//! use alfrusco::ui::{Button, Event, JsonAction, Size, Trigger, Ui, View, Window};
//!
//! let sounds = ["Glass", "Ping", "Pop", "Purr"];
//! let grid = View::grid(2, 2).spacing(8).children(sounds.map(|name| {
//!     Button::new(name).on(Trigger::Actioned, JsonAction::new().sound(name))
//! }));
//! let ui = Ui::new(Window::new(grid).size(Size::with_height(240)).padding(8));
//!
//! let json = serde_json::to_value(&ui).unwrap();
//! assert_eq!(json["window"]["root"]["layout"], "grid");
//! assert_eq!(
//!     json["window"]["root"]["components"][0]["events"][0]["action"]["src"]["sound"]["src"],
//!     "Glass"
//! );
//! ```
//!
//! # A complete window
//!
//! The example from the README:
//!
//! <!-- README example -->
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let scratch = tempfile::tempdir()?;
//! # std::env::set_current_dir(scratch.path())?;
//! use alfrusco::ui::*;
//!
//! let search = TextField::new()
//!     .id("query")
//!     .placeholder("Search fruit")
//!     .focus(true)
//!     .region(Region::Top)
//!     .on(Trigger::Changed, Action::function("fruit", function::FILTER_DATA))
//!     .on(
//!         Trigger::Actioned,
//!         JsonAction::new().finish(Finish::new().args(Placeholder::value("fruit", "selected.name.text"))),
//!     );
//!
//! let fruit = DataView::new(
//!     Cell::new(Label::empty().id("name"))
//!         .height(28)
//!         .selected("name", "colour", Colour::alfred_selected_result_text()),
//!     Data::rows(["Apple", "Banana", "Cherry"].map(|name| Row::new().value("name", "text", name)))
//!         .filter(Filter::new("name.text", Placeholder::value("query", "text"))),
//! )
//! .id("fruit")
//! .region(Region::Centre);
//!
//! let ui = Ui::new(Window::new(View::border().spacing(8).child(search).child(fruit)).padding(8));
//! std::fs::create_dir_all("fruit.alfredui")?;
//! ui.write_pretty(std::fs::File::create("fruit.alfredui/ui.json")?)?;
//! # assert!(std::path::Path::new("fruit.alfredui/ui.json").is_file());
//! # Ok(())
//! # }
//! ```
//!
//! # Keeping up with Alfred
//!
//! Alfred UI is new and its property names still change between builds.
//! Every struct here is `#[non_exhaustive]` and has a `set(key, value)`
//! method that writes any key into its JSON, so a renamed or new property is
//! usable before this crate models it. [`Component::raw`] does the same for a
//! whole component.
//!
//! # Placeholders
//!
//! Alfred replaces placeholders such as `{var:name}` and
//! `{value:field.text}` in the JSON. [`Placeholder`] builds them; it converts
//! into a `String`, so it works wherever a setter takes text.
//!
//! See <https://www.alfredapp.com/help/alfredui/>

use std::collections::BTreeMap;
use std::fmt;
use std::io;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{Arg, Result};

#[macro_use]
mod macros;
mod component;
mod dataview;
mod event;
mod json_action;
mod layout;
mod property;
mod settings;
mod window;

pub use component::{
    Border, BorderShape, Button, ButtonEffect, Checkbox, Component, FileComponent, ImageView,
    Label, LineBreak, MarkdownArea, PasswordField, Picker, PickerItem, PickerItemType, Progress,
    ProgressStyle, Template, TextAlign, TextArea, TextAreaBorder, TextBehaviour, TextField,
    TextFieldLineBreak, TextMode, TextScroll, View, WebKitView,
};
pub use dataview::{
    Cell, Data, DataView, DataViewBehaviour, DeleteRows, Filter, InsertPosition, InsertRows,
    MatchMode, Modify, ReplaceRows, Row, Selection, SelectionWrap,
};
pub use event::{function, Action, ActionType, Confirm, Event, Trigger};
pub use json_action::{
    ClipboardCopy, Dimension, Finish, Functions, Insert, JsonAction, Notification, Open,
    OpenDialog, PopupMenu, Post, Remove, Replace, Share, Sheet, Sound, SourceType, Speak, Update,
    WindowCommand,
};
pub use layout::{Constraints, Direction, GridCell, Insets, Layout, Position, Region, Stretch};
pub use property::{
    Accessibility, Colour, Font, FontSize, FontWeight, FontWidth, Image, ImageType, MenuItem,
    MenuItemState, MenuItemType, Shortcut, Size, TextInsets,
};
pub use settings::{
    DebugSettings, IntervalMode, Log, LogLevel, Pulse, VariableConfig, VariableSource,
};
pub use window::{Appearance, Toolbar, ToolbarType, Window};

/// Properties keyed `"componentid.property"` or by property name, as used by
/// [`Update::properties`].
pub type Properties = Map<String, Value>;

/// The contents of a `ui.json` file.
///
/// ```
/// use alfrusco::ui::{Colour, Label, Ui, VariableConfig, VariableSource, Window};
///
/// let ui = Ui::new(Window::new(Label::new("Hello").colour("brand")))
///     .bundle_id("com.example.greeter")
///     .colour("brand", Colour::hex("#5e2ca5"))
///     .variable_config("name", VariableConfig::new().src(VariableSource::Arg(0)));
///
/// let mut file = Vec::new();
/// ui.write_pretty(&mut file).unwrap();
/// assert!(String::from_utf8(file).unwrap().contains("\"bundleid\""));
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/ui-json/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct Ui {
    bundleid: Option<String>,
    variablesconfig: BTreeMap<String, VariableConfig>,
    variables: Map<String, Value>,
    iconpath: Option<String>,
    log: Option<Log>,
    debug: Option<DebugSettings>,
    templates: BTreeMap<String, Component>,
    colours: BTreeMap<String, Colour>,
    pulse: Option<Pulse>,
    window: Window,
    extra: Map<String, Value>,
}

impl Ui {
    /// A UI shown in `window`.
    pub fn new(window: Window) -> Self {
        Ui {
            bundleid: None,
            variablesconfig: BTreeMap::new(),
            variables: Map::new(),
            iconpath: None,
            log: None,
            debug: None,
            templates: BTreeMap::new(),
            colours: BTreeMap::new(),
            pulse: None,
            window,
            extra: Map::new(),
        }
    }

    /// The UI's bundle id. Required for persisted variables.
    ///
    /// Alfred 6.0.3 keeps persisted variables in
    /// `~/Library/Application Support/Alfred/AlfredUI/Persist/<bundleid>/`, so
    /// two installs with the same bundle id (a development and a release
    /// build, say) share them.
    pub fn bundle_id(mut self, bundle_id: impl Into<String>) -> Self {
        self.bundleid = Some(bundle_id.into());
        self
    }

    /// Configures the variable `name`.
    pub fn variable_config(mut self, name: impl Into<String>, config: VariableConfig) -> Self {
        self.variablesconfig.insert(name.into(), config);
        self
    }

    /// Sets a variable's value at start-up, readable as `{var:name}`.
    ///
    /// Not in a script's environment: pass it with `{var:name}` in args, or
    /// declare it with [`Ui::variable_config`].
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// The Dock icon when the UI runs outside Alfred.
    pub fn icon_path(mut self, path: impl Into<String>) -> Self {
        self.iconpath = Some(path.into());
        self
    }

    /// Logging.
    pub fn log(mut self, log: Log) -> Self {
        self.log = Some(log);
        self
    }

    /// Layout debugging aids.
    pub fn debug(mut self, debug: DebugSettings) -> Self {
        self.debug = Some(debug);
        self
    }

    /// Defines a template that [`Template::new`] can use by `name`.
    pub fn template(mut self, name: impl Into<String>, component: impl Into<Component>) -> Self {
        self.templates.insert(name.into(), component.into());
        self
    }

    /// Defines a colour constant usable by `name` anywhere a colour goes.
    pub fn colour(mut self, name: impl Into<String>, colour: impl Into<Colour>) -> Self {
        self.colours.insert(name.into(), colour.into());
        self
    }

    /// The pulse.
    pub fn pulse(mut self, pulse: Pulse) -> Self {
        self.pulse = Some(pulse);
        self
    }

    /// Writes compact `ui.json` to `writer`.
    pub fn write<W: io::Write>(&self, writer: W) -> Result<()> {
        Ok(serde_json::to_writer(writer, self)?)
    }

    /// Writes indented `ui.json` to `writer`, for files people also edit.
    pub fn write_pretty<W: io::Write>(&self, writer: W) -> Result<()> {
        Ok(serde_json::to_writer_pretty(writer, self)?)
    }

    extra_methods!();
}

impl Serialize for Ui {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::Error as _;
        let mut object = property::ObjectBuilder::new();
        (|| {
            object
                .some("bundleid", self.bundleid.as_ref())?
                .nonempty("variablesconfig", &self.variablesconfig)?
                .nonempty("variables", &self.variables)?
                .some("iconpath", self.iconpath.as_ref())?
                .some("log", self.log.as_ref())?
                .some("debug", self.debug.as_ref())?
                .nonempty("templates", &self.templates)?
                .nonempty("colours", &self.colours)?
                .some("pulse", self.pulse.as_ref())?
                .always("window", &self.window)?;
            Ok::<_, serde_json::Error>(())
        })()
        .map_err(S::Error::custom)?;
        object.finish(&self.extra, serializer)
    }
}

/// A placeholder that Alfred replaces when it reads the JSON or runs an
/// action.
///
/// Alfred preprocesses the JSON when it loads, replacing `var`, `constant`,
/// `file`, `file_raw` and `param` everywhere except under `bundleid`,
/// `variablesconfig`, `variables`, `args`, an action's `src`, and a
/// dataview's `data.filter`. Those keys are resolved each time they're used,
/// and they are the only places `value` and `data` work; `{var:}` there reads
/// the variable's current value.
///
/// In Alfred 6.0.3 (observed), a missing variable or component becomes an
/// empty string with a warning in the UI log, while an unknown property of an
/// existing component stays as the literal placeholder text.
///
/// ```
/// use alfrusco::ui::Placeholder;
///
/// assert_eq!(Placeholder::value("search", "text").to_string(), "{value:search.text}");
/// assert_eq!(Placeholder::var("query").as_str(), "{var:query}");
/// let greeting = format!("Hello {}", Placeholder::var("name"));
/// assert_eq!(greeting, "Hello {var:name}");
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/placeholders/>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Placeholder(String);

impl Placeholder {
    fn of(kind: &str, name: impl fmt::Display) -> Self {
        Placeholder(format!("{{{kind}:{name}}}"))
    }

    /// `{var:name}`: a variable's value.
    pub fn var(name: impl fmt::Display) -> Self {
        Self::of("var", name)
    }

    /// `{const:name}`: a workflow constant.
    pub fn constant(name: impl fmt::Display) -> Self {
        Self::of("const", name)
    }

    /// `{file:path}`: the contents of a file, processed as part of the JSON.
    pub fn file(path: impl fmt::Display) -> Self {
        Self::of("file", path)
    }

    /// `{fileraw:path}`: the contents of a file, not processed further. Faster
    /// for large data files.
    pub fn file_raw(path: impl fmt::Display) -> Self {
        Self::of("fileraw", path)
    }

    /// `{param:name}`: a template parameter.
    pub fn param(name: impl fmt::Display) -> Self {
        Self::of("param", name)
    }

    /// `{value:id.property}`: a component property, read when an action runs.
    /// `property` may be a path such as `selected.value`.
    pub fn value(id: impl fmt::Display, property: impl fmt::Display) -> Self {
        Self::of("value", format_args!("{id}.{property}"))
    }

    /// `{value:self.property}`: a property of the component whose event runs
    /// the action.
    pub fn value_of_self(property: impl fmt::Display) -> Self {
        Self::value("self", property)
    }

    /// `{data:property}`: a value from the dataview row the action runs for.
    pub fn data(property: impl fmt::Display) -> Self {
        Self::of("data", property)
    }

    /// The placeholder text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Placeholder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Placeholder {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl From<Placeholder> for String {
    fn from(placeholder: Placeholder) -> Self {
        placeholder.0
    }
}

impl From<Placeholder> for Value {
    fn from(placeholder: Placeholder) -> Self {
        Value::String(placeholder.0)
    }
}

impl From<Placeholder> for Arg {
    fn from(placeholder: Placeholder) -> Self {
        Arg::One(placeholder.0)
    }
}

impl From<Placeholder> for Colour {
    fn from(placeholder: Placeholder) -> Self {
        Colour::new(placeholder.0)
    }
}
