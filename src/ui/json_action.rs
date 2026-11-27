//! JSON actions: the commands that change a running UI.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

use super::property::{arg_to_value, one_or_many, push_arg, ObjectBuilder};
use super::{Action, Component, MenuItem, Properties, Pulse};
use crate::Arg;

/// A set of JSON action commands, run by an [`Action::json`] action or
/// printed by a script or the pulse.
///
/// Alfred runs the commands in a fixed order, whatever order they're set in:
/// `remove`, `insert`, `update`, `replace`, `sheet`, `share`, `sound`, `speak`,
/// `copy`, `open`, `opendialog`, `window`, `notification`, `functions`,
/// `popupmenu`, `finish`. `variables` are set first, so commands can use them.
///
/// ```
/// use alfrusco::ui::{JsonAction, Placeholder, Update};
///
/// let actions = JsonAction::new()
///     .variable("words", Placeholder::value("notes", "text"))
///     .speak(Placeholder::var("words"))
///     .update(Update::new().property("status", "text", "Spoken"));
/// # let _ = actions;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/actions/json/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JsonAction {
    pub(crate) variables: Map<String, Value>,
    pub(crate) pulse: Option<Pulse>,
    pub(crate) remove: Option<Remove>,
    pub(crate) insert: Option<Insert>,
    pub(crate) update: Option<Update>,
    pub(crate) replace: Option<Replace>,
    pub(crate) sheet: Option<Sheet>,
    pub(crate) share: Option<Share>,
    pub(crate) sound: Option<Sound>,
    pub(crate) speak: Option<Speak>,
    pub(crate) copy: Option<ClipboardCopy>,
    pub(crate) open: Option<Open>,
    pub(crate) opendialog: Option<OpenDialog>,
    pub(crate) window: Option<WindowCommand>,
    pub(crate) notification: Option<Notification>,
    pub(crate) functions: Option<Functions>,
    pub(crate) popupmenu: Option<PopupMenu>,
    pub(crate) finish: Option<Finish>,
    pub(crate) extra: Map<String, Value>,
}

impl JsonAction {
    /// No commands.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a variable. Put `{value:id.property}` placeholders here to use a
    /// component's value in the commands, as `{var:name}`.
    ///
    /// Not in a script's environment: pass it with `{var:name}` in args, or
    /// declare it with [`Ui::variable_config`](super::Ui::variable_config).
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Reconfigures the pulse.
    pub fn pulse(mut self, pulse: Pulse) -> Self {
        self.pulse = Some(pulse);
        self
    }

    /// Removes components.
    pub fn remove(mut self, remove: Remove) -> Self {
        self.remove = Some(remove);
        self
    }

    /// Inserts components.
    pub fn insert(mut self, insert: Insert) -> Self {
        self.insert = Some(insert);
        self
    }

    /// Updates component properties.
    pub fn update(mut self, update: Update) -> Self {
        self.update = Some(update);
        self
    }

    /// Replaces components.
    pub fn replace(mut self, replace: Replace) -> Self {
        self.replace = Some(replace);
        self
    }

    /// Shows or closes a sheet.
    pub fn sheet(mut self, sheet: Sheet) -> Self {
        self.sheet = Some(sheet);
        self
    }

    /// Shows the macOS share sheet.
    pub fn share(mut self, share: Share) -> Self {
        self.share = Some(share);
        self
    }

    /// Plays a sound: a system sound name such as `"Glass"`, a sound file, or
    /// a [`Sound`].
    pub fn sound(mut self, sound: impl Into<Sound>) -> Self {
        self.sound = Some(sound.into());
        self
    }

    /// Speaks text aloud: a string or a [`Speak`].
    pub fn speak(mut self, speak: impl Into<Speak>) -> Self {
        self.speak = Some(speak.into());
        self
    }

    /// Copies text, files or URLs to the clipboard.
    pub fn copy(mut self, copy: ClipboardCopy) -> Self {
        self.copy = Some(copy);
        self
    }

    /// Opens files or URLs.
    pub fn open(mut self, open: Open) -> Self {
        self.open = Some(open);
        self
    }

    /// Shows a macOS Open dialog.
    pub fn open_dialog(mut self, dialog: OpenDialog) -> Self {
        self.opendialog = Some(dialog);
        self
    }

    /// Resizes the window.
    pub fn window(mut self, window: WindowCommand) -> Self {
        self.window = Some(window);
        self
    }

    /// Posts or removes notifications.
    pub fn notification(mut self, notification: Notification) -> Self {
        self.notification = Some(notification);
        self
    }

    /// Runs functions on components.
    pub fn functions(mut self, functions: Functions) -> Self {
        self.functions = Some(functions);
        self
    }

    /// Shows a popup menu on a component.
    pub fn popup_menu(mut self, menu: PopupMenu) -> Self {
        self.popupmenu = Some(menu);
        self
    }

    /// Finishes the UI: closes it and, when the UI opened inside Alfred, passes
    /// args and variables to the next connected object. See [`Finish`].
    pub fn finish(mut self, finish: Finish) -> Self {
        self.finish = Some(finish);
        self
    }

    extra_methods!();
}

impl Serialize for JsonAction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error as _;
        let mut object = ObjectBuilder::new();
        (|| {
            object
                .nonempty("variables", &self.variables)?
                .some("pulse", self.pulse.as_ref())?
                .some("remove", self.remove.as_ref())?
                .some("insert", self.insert.as_ref())?
                .some("update", self.update.as_ref())?
                .some("replace", self.replace.as_ref())?
                .some("sheet", self.sheet.as_ref())?
                .some("share", self.share.as_ref())?
                .some("sound", self.sound.as_ref())?
                .some("speak", self.speak.as_ref())?
                .some("copy", self.copy.as_ref())?
                .some("open", self.open.as_ref())?
                .some("opendialog", self.opendialog.as_ref())?
                .some("window", self.window.as_ref())?
                .some("notification", self.notification.as_ref())?
                .some("functions", self.functions.as_ref())?
                .some("popupmenu", self.popupmenu.as_ref())?
                .some("finish", self.finish.as_ref())?;
            Ok::<_, serde_json::Error>(())
        })()
        .map_err(S::Error::custom)?;
        object.finish(&self.extra, serializer)
    }
}

/// Removes components by id. Alfred then lays out the whole UI again.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Remove {
    ids: Arg,
    #[serde(skip_serializing_if = "Option::is_none")]
    relayout: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Remove {
    /// Removes the component or components with `ids`.
    pub fn new(ids: impl Into<Arg>) -> Self {
        Remove {
            ids: ids.into(),
            relayout: None,
            extra: Map::new(),
        }
    }

    /// Marks the layout for recalculation.
    pub fn relayout(mut self, relayout: bool) -> Self {
        self.relayout = Some(relayout);
        self
    }

    extra_methods!();
}

/// Appends components to views by the parent view's id.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Insert {
    ids: BTreeMap<String, Vec<Component>>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Insert {
    /// Nothing to insert yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends `component` to the view with id `parent`.
    pub fn component(mut self, parent: impl Into<String>, component: impl Into<Component>) -> Self {
        self.ids
            .entry(parent.into())
            .or_default()
            .push(component.into());
        self
    }

    extra_methods!();
}

/// Changes properties of components by id.
///
/// In Alfred 6.0.3 (observed), an `update` that sets a text field's text
/// doesn't fire its `changed` event, while a `functions` call such as
/// `selectnext` on a dataview does fire the dataview's `changed` event.
///
/// ```
/// use alfrusco::ui::{Colour, Update};
///
/// let update = Update::new()
///     .property("status", "text", "Synced")
///     .property("status", "colour", Colour::named("systemGreenColor"))
///     .relayout(true);
/// # let _ = update;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/actions/json/update/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Update {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) relayout: Option<bool>,
    pub(crate) ids: BTreeMap<String, Properties>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Update {
    /// Nothing to update yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `property` to `value` on the component with id `id`.
    pub fn property(
        mut self,
        id: impl Into<String>,
        property: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        self.ids
            .entry(id.into())
            .or_default()
            .insert(property.into(), value.into());
        self
    }

    /// Replaces the data of the dataview with id `id`, for example its rows.
    pub fn data(self, id: impl Into<String>, data: super::Data) -> Self {
        self.property(id, "data", data)
    }

    /// Sets several properties on the component with id `id`.
    pub fn properties(mut self, id: impl Into<String>, properties: Properties) -> Self {
        self.ids.entry(id.into()).or_default().extend(properties);
        self
    }

    /// When true, Alfred lays out the UI again. Set it when an update changes
    /// a component's size.
    pub fn relayout(mut self, relayout: bool) -> Self {
        self.relayout = Some(relayout);
        self
    }

    extra_methods!();
}

/// Replaces whole components by id. Alfred then lays out the whole UI again.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Replace {
    #[serde(skip_serializing_if = "Option::is_none")]
    retainids: Option<bool>,
    ids: BTreeMap<String, Component>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Replace {
    /// Nothing to replace yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the component with id `id` by `component`.
    pub fn component(mut self, id: impl Into<String>, component: impl Into<Component>) -> Self {
        self.ids.insert(id.into(), component.into());
        self
    }

    /// When true (Alfred's default), the new component keeps the old one's id
    /// unless it sets its own.
    pub fn retain_ids(mut self, retain: bool) -> Self {
        self.retainids = Some(retain);
        self
    }

    extra_methods!();
}

/// Shows a modal sheet over the UI, or closes one by id.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Sheet {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    close: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closeonescape: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root: Option<Box<Component>>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Sheet {
    /// Shows a sheet whose content is `root`.
    pub fn show(root: impl Into<Component>) -> Self {
        Sheet {
            root: Some(Box::new(root.into())),
            ..Self::default()
        }
    }

    /// Closes the sheet with id `id`.
    pub fn close(id: impl Into<String>) -> Self {
        Sheet {
            close: Some(id.into()),
            ..Self::default()
        }
    }

    /// The sheet's id, so a later action can close it.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// When true (Alfred's default), Escape closes the sheet. Set an id when
    /// turning this off.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.closeonescape = Some(close);
        self
    }

    extra_methods!();
}

/// Whether a share, copy or open source is a file path, URL or text.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    /// Plain text. Only `copy` accepts it, and it's `copy`'s default.
    Text,
    /// A file path.
    File,
    /// A URL.
    Url,
}

macro_rules! sourced_command {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[non_exhaustive]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
        pub struct $name {
            #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
            kind: Option<SourceType>,
            src: Arg,
            #[serde(flatten)]
            extra: Map<String, Value>,
        }

        impl $name {
            /// One or more sources of type `kind`.
            pub fn new(kind: SourceType, src: impl Into<Arg>) -> Self {
                $name {
                    kind: Some(kind),
                    src: src.into(),
                    extra: Map::new(),
                }
            }

            /// One or more file paths.
            pub fn files(src: impl Into<Arg>) -> Self {
                Self::new(SourceType::File, src)
            }

            /// One or more URLs.
            pub fn urls(src: impl Into<Arg>) -> Self {
                Self::new(SourceType::Url, src)
            }

            extra_methods!();
        }
    };
}

sourced_command!(
    /// Shows the macOS share sheet for files or URLs.
    Share
);
sourced_command!(
    /// Opens files or URLs with their default apps.
    Open
);
sourced_command!(
    /// Copies text, file references or URLs to the clipboard.
    ClipboardCopy
);

impl ClipboardCopy {
    /// Copies `text` as plain text.
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(SourceType::Text, text.into())
    }
}

/// Plays a sound.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sound {
    src: String,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Sound {
    /// A system sound name such as `"Glass"`, or a sound file path relative to
    /// the `.alfredui` folder or absolute.
    pub fn new(src: impl Into<String>) -> Self {
        Sound {
            src: src.into(),
            extra: Map::new(),
        }
    }

    extra_methods!();
}

impl From<&str> for Sound {
    fn from(src: &str) -> Self {
        Sound::new(src)
    }
}

impl From<String> for Sound {
    fn from(src: String) -> Self {
        Sound::new(src)
    }
}

impl From<super::Placeholder> for Sound {
    fn from(src: super::Placeholder) -> Self {
        Sound::new(src)
    }
}

/// Speaks text aloud.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Speak {
    text: String,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Speak {
    /// Speaks `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Speak {
            text: text.into(),
            extra: Map::new(),
        }
    }

    extra_methods!();
}

impl From<&str> for Speak {
    fn from(text: &str) -> Self {
        Speak::new(text)
    }
}

impl From<String> for Speak {
    fn from(text: String) -> Self {
        Speak::new(text)
    }
}

impl From<super::Placeholder> for Speak {
    fn from(text: super::Placeholder) -> Self {
        Speak::new(text)
    }
}

/// A macOS Open dialog. The chosen path or paths go into a variable, and the
/// dialog's actions run when the user confirms.
///
/// ```
/// use alfrusco::ui::{JsonAction, Open, OpenDialog, Placeholder};
///
/// let pick = OpenDialog::new()
///     .file_types(["public.image"])
///     .multiselect(true)
///     .set_variable("picked")
///     .action(JsonAction::new().open(Open::files(Placeholder::var("picked"))));
/// # let _ = pick;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct OpenDialog {
    #[serde(skip_serializing_if = "Option::is_none")]
    canchoosefiles: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canchoosefolders: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filetypes: Option<Arg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiselect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setvariable: Option<String>,
    #[serde(serialize_with = "one_or_many", skip_serializing_if = "Vec::is_empty")]
    action: Vec<Action>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl OpenDialog {
    /// A dialog with Alfred's defaults: one file, no folders.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether files can be chosen. Alfred's default is true.
    pub fn can_choose_files(mut self, allow: bool) -> Self {
        self.canchoosefiles = Some(allow);
        self
    }

    /// Whether folders can be chosen. Alfred's default is false.
    pub fn can_choose_folders(mut self, allow: bool) -> Self {
        self.canchoosefolders = Some(allow);
        self
    }

    /// The uniform type identifiers that can be chosen, such as `public.png`.
    pub fn file_types(mut self, types: impl Into<Arg>) -> Self {
        self.filetypes = Some(types.into());
        self
    }

    /// The folder the dialog opens in.
    pub fn folder(mut self, folder: impl Into<String>) -> Self {
        self.folder = Some(folder.into());
        self
    }

    /// The confirm button's text.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// A short message at the top of the dialog.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Whether several files can be chosen.
    pub fn multiselect(mut self, multiselect: bool) -> Self {
        self.multiselect = Some(multiselect);
        self
    }

    /// The variable that receives the chosen path, or an array of paths when
    /// `multiselect` is on.
    pub fn set_variable(mut self, name: impl Into<String>) -> Self {
        self.setvariable = Some(name.into());
        self
    }

    /// Adds an action to run when the user confirms.
    pub fn action(mut self, action: impl Into<Action>) -> Self {
        self.action.push(action.into());
        self
    }

    extra_methods!();
}

/// Window controls run as a JSON action.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct WindowCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    resize: Option<Resize>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
struct Resize {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<Dimension>,
}

/// A content width or height for [`WindowCommand::resize`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    /// A size in pixels.
    Pixels(u32),
    /// Calculated from the content.
    Auto,
}

impl Serialize for Dimension {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Dimension::Pixels(pixels) => serializer.serialize_u32(*pixels),
            Dimension::Auto => serializer.serialize_str("auto"),
        }
    }
}

impl From<u32> for Dimension {
    fn from(pixels: u32) -> Self {
        Dimension::Pixels(pixels)
    }
}

impl WindowCommand {
    /// Resizes the window's content. `None` keeps the current width or height.
    pub fn resize(width: Option<Dimension>, height: Option<Dimension>) -> Self {
        WindowCommand {
            resize: Some(Resize { width, height }),
            extra: Map::new(),
        }
    }

    extra_methods!();
}

/// Posts notifications to Notification Centre or removes them.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Notification {
    #[serde(serialize_with = "one_or_many", skip_serializing_if = "Vec::is_empty")]
    pub(crate) post: Vec<Post>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Arg>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Notification {
    /// No notifications yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a notification to post.
    pub fn post(mut self, post: Post) -> Self {
        self.post.push(post);
        self
    }

    /// Removes the notifications with these uids.
    pub fn remove(mut self, uids: impl Into<Arg>) -> Self {
        self.remove = Some(uids.into());
        self
    }

    extra_methods!();
}

/// One notification to post.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Post {
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Post {
    /// A notification with a title and text.
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Self {
        Post {
            title: Some(title.into()),
            text: Some(text.into()),
            ..Self::default()
        }
    }

    /// A uid, so a later notification with the same uid replaces this one.
    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    extra_methods!();
}

/// Runs functions on components by id.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Functions {
    pub(crate) ids: BTreeMap<String, Arg>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Functions {
    /// No functions yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs one or more functions, such as
    /// [`function::CARET_START`](super::function::CARET_START), on the
    /// component with id `id`.
    pub fn call(mut self, id: impl Into<String>, functions: impl Into<Arg>) -> Self {
        self.ids.insert(id.into(), functions.into());
        self
    }

    extra_methods!();
}

/// A popup menu shown on a component.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PopupMenu {
    src: String,
    menu: Vec<MenuItem>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl PopupMenu {
    /// A menu shown on the component with id `component`.
    pub fn new(component: impl Into<String>) -> Self {
        PopupMenu {
            src: component.into(),
            menu: Vec::new(),
            extra: Map::new(),
        }
    }

    /// Adds a menu item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.menu.push(item);
        self
    }

    extra_methods!();
}

/// Closes the UI. Inside a workflow, its args and variables go to the next
/// connected object.
///
/// What Alfred 6.0.3 does (observed):
///
/// - Output reaches the next workflow object only when the Alfred UI object
///   opens inside Alfred (launch mode 0). A UI launched externally just
///   closes, and nothing downstream runs.
/// - Object and array variable values are dropped before the next object;
///   send strings or numbers.
/// - Actions after `finish` in the same array still run, then the UI quits.
///
/// ```
/// use alfrusco::ui::{Finish, Placeholder};
///
/// let finish = Finish::new()
///     .args(Placeholder::value("results", "selected.name.text"))
///     .variable("source", "picker");
/// # let _ = finish;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/actions/json/finish/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Finish {
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Value>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    variables: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flattenarrayargs: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Finish {
    /// Finishes with no args or variables.
    pub fn new() -> Self {
        Self::default()
    }

    /// The arg or args to pass on. Each array element becomes a separate
    /// argument of the next object.
    pub fn args(mut self, args: impl Into<Arg>) -> Self {
        self.args = Some(arg_to_value(args.into()));
        self
    }

    /// Adds one argument after any already set. It may be any JSON value.
    pub fn arg(mut self, arg: impl Into<Value>) -> Self {
        push_arg(&mut self.args, arg.into());
        self
    }

    /// A variable to merge into the workflow's variables.
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// When true, arrays among the args are flattened into one list.
    pub fn flatten_array_args(mut self, flatten: bool) -> Self {
        self.flattenarrayargs = Some(flatten);
        self
    }

    extra_methods!();
}
