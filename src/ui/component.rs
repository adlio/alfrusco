//! The components a UI is built from.

use serde::Serialize;
use serde_json::{Map, Value};

use super::property::Common;
use super::{
    Appearance, Colour, DataView, Direction, Font, Image, Layout, Placeholder, Shortcut, TextInsets,
};
use crate::IconResize;

/// Any Alfred UI component. Each component type converts into it with
/// `From`, so builders take `impl Into<Component>`.
///
/// [`Component::raw`] holds JSON this crate doesn't model, and
/// [`Component::placeholder`] holds a whole-component placeholder such as
/// `{file:components/sidebar.json}`.
///
/// See <https://www.alfredapp.com/help/alfredui/components/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[allow(missing_docs)]
// A UI tree is built once and serialized, so variant size doesn't matter, and
// boxing every variant would make matching on components clumsier.
#[allow(clippy::large_enum_variant)]
pub enum Component {
    View(View),
    Label(Label),
    Button(Button),
    Checkbox(Checkbox),
    Picker(Picker),
    TextField(TextField),
    PasswordField(PasswordField),
    TextArea(TextArea),
    MarkdownArea(MarkdownArea),
    Progress(Progress),
    ImageView(ImageView),
    DataView(DataView),
    WebKitView(WebKitView),
    Template(Template),
    File(FileComponent),
    /// JSON written as is.
    #[serde(untagged)]
    Raw(Value),
}

impl Component {
    /// A component written as the given JSON, for component types or shapes
    /// this crate doesn't model.
    pub fn raw(json: Value) -> Self {
        Component::Raw(json)
    }

    /// A placeholder that Alfred replaces with a whole component, such as
    /// [`Placeholder::file`].
    pub fn placeholder(placeholder: impl Into<String>) -> Self {
        Component::Raw(Value::String(placeholder.into()))
    }
}

impl From<Placeholder> for Component {
    fn from(placeholder: Placeholder) -> Self {
        Component::placeholder(placeholder)
    }
}

macro_rules! component_from {
    ($($ty:ident),* $(,)?) => {
        $(
            impl From<$ty> for Component {
                fn from(component: $ty) -> Self {
                    Component::$ty(component)
                }
            }
        )*
    };
}

component_from!(
    View,
    Label,
    Button,
    Checkbox,
    Picker,
    TextField,
    PasswordField,
    TextArea,
    MarkdownArea,
    Progress,
    ImageView,
    DataView,
    WebKitView,
    Template,
);

impl From<FileComponent> for Component {
    fn from(component: FileComponent) -> Self {
        Component::File(component)
    }
}

/// A container that arranges its components with a [`Layout`].
///
/// Layout properties live on the view itself: `direction` for inline,
/// `rows` and `cols` for grid and flex, and `padding` and `spacing` for all.
///
/// ```
/// use alfrusco::ui::{Button, Label, Region, View};
///
/// let view = View::border()
///     .padding(8)
///     .spacing(4)
///     .child(Label::new("Name").region(Region::Top))
///     .child(Button::new("Save").region(Region::Bottom));
/// # let _ = view;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/components/view/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct View {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<Layout>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rows: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cols: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cliptobounds: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cornerradius: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<Border>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focusable: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    shortcuts: Vec<Shortcut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vetomovablebybackground: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<Component>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl View {
    /// A view with no layout set, which Alfred treats as a stack layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// A view with the given layout.
    pub fn with_layout(layout: Layout) -> Self {
        Self::new().layout(layout)
    }

    /// A stack layout: every component fills the view, later ones on top.
    pub fn stack() -> Self {
        Self::with_layout(Layout::Stack)
    }

    /// A border layout: give each component a [`Region`](super::Region).
    pub fn border() -> Self {
        Self::with_layout(Layout::Border)
    }

    /// An equal-sized grid, filled row by row from the top left.
    pub fn grid(rows: u32, cols: u32) -> Self {
        Self::with_layout(Layout::Grid).rows(rows).cols(cols)
    }

    /// An inline layout in `direction`.
    pub fn inline(direction: Direction) -> Self {
        Self::with_layout(Layout::Inline).direction(direction)
    }

    /// An inline layout, left to right.
    pub fn horizontal() -> Self {
        Self::inline(Direction::Horizontal)
    }

    /// An inline layout, top to bottom.
    pub fn vertical() -> Self {
        Self::inline(Direction::Vertical)
    }

    /// A form layout: components pair up into a label column and a field
    /// column.
    pub fn form() -> Self {
        Self::with_layout(Layout::Form)
    }

    /// A flex grid: give each component a [`GridCell`](super::GridCell).
    pub fn flex(rows: u32, cols: u32) -> Self {
        Self::with_layout(Layout::Flex).rows(rows).cols(cols)
    }

    /// The layout.
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// The direction of an inline layout.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Rows in a grid or flex layout.
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Columns in a grid or flex layout.
    pub fn cols(mut self, cols: u32) -> Self {
        self.cols = Some(cols);
        self
    }

    /// Space inside the view's edges, 0-64 pixels.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Space between components, 0-64 pixels.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Clips content to the view's bounds.
    pub fn clip_to_bounds(mut self, clip: bool) -> Self {
        self.cliptobounds = Some(clip);
        self
    }

    /// Corner radius, 0-32 pixels.
    pub fn corner_radius(mut self, radius: u32) -> Self {
        self.cornerradius = Some(radius);
        self
    }

    /// Forces light or dark mode for this view and everything in it. The
    /// default is [`Appearance::Inherit`].
    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// A border drawn inside the view's edges (the `border` property).
    #[doc(alias = "border")]
    pub fn outline(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Lets the view take keyboard focus like a component.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = Some(focusable);
        self
    }

    /// Adds a keyboard shortcut that works while focus is inside this view.
    pub fn view_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcuts.push(shortcut);
        self
    }

    /// Stops the window being dragged by this view's background.
    pub fn veto_movable_by_background(mut self, veto: bool) -> Self {
        self.vetomovablebybackground = Some(veto);
        self
    }

    /// Adds a component.
    pub fn child(mut self, component: impl Into<Component>) -> Self {
        self.components.push(component.into());
        self
    }

    /// Adds several components.
    pub fn children<C: Into<Component>>(mut self, components: impl IntoIterator<Item = C>) -> Self {
        self.components
            .extend(components.into_iter().map(Into::into));
        self
    }

    common_methods!();
}

/// A border drawn inside a view's edges.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Border {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Border {
    /// A border `width` pixels wide (0-32) in `colour`.
    pub fn new(width: u32, colour: impl Into<Colour>) -> Self {
        Border {
            width: Some(width),
            colour: Some(colour.into()),
            extra: Map::new(),
        }
    }

    extra_methods!();
}

/// How a label truncates or wraps text that doesn't fit.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineBreak {
    /// One line, cut at the start.
    TruncateHead,
    /// One line, cut in the middle.
    TruncateMiddle,
    /// One line, cut at the end. This is the default.
    TruncateTail,
    /// Wrap at word boundaries.
    WordWrap,
    /// Wrap at any character.
    CharWrap,
    /// One line, clipped.
    Clip,
}

/// Text alignment.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum TextAlign {
    Left,
    Right,
    #[doc(alias = "Center")]
    Centre,
    Justify,
    /// Based on the text's writing direction. This is the default.
    Natural,
}

/// Read-only text such as a title, status or description.
///
/// ```
/// use alfrusco::ui::{Colour, Font, FontSize, Label, TextAlign};
///
/// let title = Label::new("Colours")
///     .font(Font::new().size(FontSize::Large))
///     .colour(Colour::named("secondaryLabelColor"))
///     .text_align(TextAlign::Centre);
/// # let _ = title;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/components/label/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Label {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selectable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    linebreak: Option<LineBreak>,
    #[serde(skip_serializing_if = "Option::is_none")]
    textalign: Option<TextAlign>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Label {
    /// A label showing `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self::empty().text(text)
    }

    /// A label with no text yet, such as one a dataview row fills in.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// The text colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Whether the text can be selected.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = Some(selectable);
        self
    }

    /// How text that doesn't fit is truncated or wrapped.
    pub fn line_break(mut self, line_break: LineBreak) -> Self {
        self.linebreak = Some(line_break);
        self
    }

    /// The text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.textalign = Some(align);
        self
    }

    common_methods!();
}

/// The shape of a bordered button or picker. Requires macOS 26.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderShape {
    /// Chosen by macOS. This is the default.
    Auto,
    /// A capsule.
    Capsule,
    /// A rounded rectangle.
    RoundedRectangle,
    /// A circle. Buttons only.
    Circle,
}

/// A button's visual effect. Requires macOS 26.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonEffect {
    /// The standard macOS style.
    Default,
    /// The glass style.
    Glass,
}

/// A push button. Clicking it fires [`Trigger::Actioned`](super::Trigger).
///
/// ```
/// use alfrusco::ui::{Button, Image, JsonAction, Trigger};
///
/// let refresh = Button::new("Refresh")
///     .image(Image::symbol("arrow.clockwise"))
///     .bordered(false)
///     .shortcut("⌘R")
///     .on(Trigger::Actioned, JsonAction::new().sound("Pop"));
/// # let _ = refresh;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/components/button/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Button {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordershape: Option<BorderShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect: Option<ButtonEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contenttintcolour: Option<Colour>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Button {
    /// A button labelled `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self::default().text(text)
    }

    /// A button showing only `image`.
    pub fn with_image(image: Image) -> Self {
        Self::default().image(image)
    }

    /// The button text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// An image on the button.
    pub fn image(mut self, image: Image) -> Self {
        self.image = Some(image);
        self
    }

    /// The font of the button text.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Whether the button has a border. Alfred's default is true.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = Some(bordered);
        self
    }

    /// The border's shape when bordered.
    pub fn border_shape(mut self, shape: BorderShape) -> Self {
        self.bordershape = Some(shape);
        self
    }

    /// The button's visual effect.
    pub fn effect(mut self, effect: ButtonEffect) -> Self {
        self.effect = Some(effect);
        self
    }

    /// A tint for the content of a borderless button.
    pub fn content_tint_colour(mut self, colour: impl Into<Colour>) -> Self {
        self.contenttintcolour = Some(colour.into());
        self
    }

    common_methods!();
}

/// An on/off checkbox. A change fires [`Trigger::Changed`](super::Trigger).
///
/// See <https://www.alfredapp.com/help/alfredui/components/checkbox/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Checkbox {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    checked: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Checkbox {
    /// A checkbox labelled `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self::default().text(text)
    }

    /// The label to the right of the box.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Whether the box is ticked.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    common_methods!();
}

/// A popup list to choose one item from. A change fires
/// [`Trigger::Changed`](super::Trigger); read the choice with
/// `{value:id.selected.value}`.
///
/// ```
/// use alfrusco::ui::{Picker, PickerItem};
///
/// let units = Picker::new()
///     .id("units")
///     .item(PickerItem::new("Metric", "metric"))
///     .item(PickerItem::separator())
///     .item(PickerItem::new("Imperial", "imperial"))
///     .selected("metric");
/// # let _ = units;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/components/picker/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Picker {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<PickerItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordershape: Option<BorderShape>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Picker {
    /// An empty picker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an item.
    pub fn item(mut self, item: PickerItem) -> Self {
        self.items.push(item);
        self
    }

    /// Selects the item with this value.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Whether the picker has a border. Alfred's default is true.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = Some(bordered);
        self
    }

    /// The border's shape when bordered.
    pub fn border_shape(mut self, shape: BorderShape) -> Self {
        self.bordershape = Some(shape);
        self
    }

    common_methods!();
}

/// One entry in a [`Picker`].
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PickerItem {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    kind: Option<PickerItemType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// The kind of a [`PickerItem`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PickerItemType {
    /// A choosable item. This is the default.
    Item,
    /// A separator line.
    Separator,
}

impl PickerItem {
    /// An item showing `text` whose value is `value`. Values should be unique.
    pub fn new(text: impl Into<String>, value: impl Into<String>) -> Self {
        PickerItem {
            text: Some(text.into()),
            value: Some(value.into()),
            ..Self::default()
        }
    }

    /// A separator line.
    pub fn separator() -> Self {
        PickerItem {
            kind: Some(PickerItemType::Separator),
            ..Self::default()
        }
    }

    extra_methods!();
}

/// What a text field does when text reaches the end of the line.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextFieldLineBreak {
    /// Scroll sideways. This is the default.
    Scroll,
    /// Wrap onto a new line.
    Wrap,
}

/// An editable single-line text input.
///
/// Fires [`Trigger::Actioned`](super::Trigger) on return,
/// [`Trigger::Changed`](super::Trigger) on each edit and
/// [`Trigger::TypingStopped`](super::Trigger) after a pause.
///
/// See <https://www.alfredapp.com/help/alfredui/components/textfield/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TextField {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholdercolour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    textinsets: Option<TextInsets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focusring: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    editable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selectable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    linebreak: Option<TextFieldLineBreak>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl TextField {
    /// An empty text field.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Text shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Allows newlines with ⌥↩ or ⇧↩.
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = Some(multiline);
        self
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// The text colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// The placeholder text colour.
    pub fn placeholder_colour(mut self, colour: impl Into<Colour>) -> Self {
        self.placeholdercolour = Some(colour.into());
        self
    }

    /// Whether the field has a border. Alfred's default is true.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = Some(bordered);
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// Space around the text.
    pub fn text_insets(mut self, insets: TextInsets) -> Self {
        self.textinsets = Some(insets);
        self
    }

    /// Whether the focus ring is drawn. Alfred's default is true.
    pub fn focus_ring(mut self, focus_ring: bool) -> Self {
        self.focusring = Some(focus_ring);
        self
    }

    /// Whether the text can be edited.
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = Some(editable);
        self
    }

    /// Whether the text can be selected.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = Some(selectable);
        self
    }

    /// What happens when text reaches the end of the line.
    pub fn line_break(mut self, line_break: TextFieldLineBreak) -> Self {
        self.linebreak = Some(line_break);
        self
    }

    common_methods!();
}

/// A text field that hides what's typed. Read it with
/// `{value:id.text}`.
///
/// There is no `text` setter: don't put a password into the UI JSON. If you
/// must prefill one, use `set("text", ...)` and keep the JSON out of version
/// control and logs.
///
/// See <https://www.alfredapp.com/help/alfredui/components/passwordfield/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PasswordField {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    textinsets: Option<TextInsets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focusring: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl PasswordField {
    /// An empty password field.
    pub fn new() -> Self {
        Self::default()
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// The text colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// Whether the field has a border. Alfred's default is true.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = Some(bordered);
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// Space around the text.
    pub fn text_insets(mut self, insets: TextInsets) -> Self {
        self.textinsets = Some(insets);
        self
    }

    /// Whether the focus ring is drawn. Alfred's default is true.
    pub fn focus_ring(mut self, focus_ring: bool) -> Self {
        self.focusring = Some(focus_ring);
        self
    }

    common_methods!();
}

/// How new text set on a text area or markdown area combines with the text
/// already there.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextMode {
    /// Replace all the text. This is the default.
    Replace,
    /// Add to the end.
    Append,
    /// Add to the start.
    Prepend,
    /// Replace only the text set last time.
    ReplaceLast,
}

/// Where a text area or markdown area scrolls after text is set.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextScroll {
    /// Don't scroll. This is the default.
    None,
    /// Scroll automatically.
    Auto,
    /// Scroll to the start.
    Start,
    /// Scroll to the end.
    End,
}

/// How a text area or markdown area handles text set by the UI or an
/// action.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TextBehaviour {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<TextMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scroll: Option<TextScroll>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl TextBehaviour {
    /// Alfred's defaults: replace, no scroll.
    pub fn new() -> Self {
        Self::default()
    }

    /// How new text combines with the old.
    pub fn mode(mut self, mode: TextMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Where to scroll after setting text.
    pub fn scroll(mut self, scroll: TextScroll) -> Self {
        self.scroll = Some(scroll);
        self
    }

    extra_methods!();
}

/// The border of a [`TextArea`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAreaBorder {
    /// A bezelled border.
    Bezel,
    /// A solid line.
    Solid,
    /// No border. This is the default.
    None,
}

/// A scrolling, multi-line text view, editable or read-only.
///
/// See <https://www.alfredapp.com/help/alfredui/components/textarea/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TextArea {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bordertype: Option<TextAreaBorder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    editable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selectable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behaviour: Option<TextBehaviour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    autoadjustcontentinsets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    textinsets: Option<TextInsets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disableallsubstitutions: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl TextArea {
    /// An empty text area.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// The text colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// The border.
    pub fn border_type(mut self, border: TextAreaBorder) -> Self {
        self.bordertype = Some(border);
        self
    }

    /// Whether the text can be edited.
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = Some(editable);
        self
    }

    /// Whether the text can be selected.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = Some(selectable);
        self
    }

    /// How text set later combines with the text already there.
    pub fn behaviour(mut self, behaviour: TextBehaviour) -> Self {
        self.behaviour = Some(behaviour);
        self
    }

    /// Whether scroll insets adjust for a title bar overlapping the content.
    /// Alfred's default is true.
    pub fn auto_adjust_content_insets(mut self, adjust: bool) -> Self {
        self.autoadjustcontentinsets = Some(adjust);
        self
    }

    /// Space around the text.
    pub fn text_insets(mut self, insets: TextInsets) -> Self {
        self.textinsets = Some(insets);
        self
    }

    /// When true, turns off smart quotes, dashes, spell checking and other
    /// substitutions.
    pub fn disable_all_substitutions(mut self, disable: bool) -> Self {
        self.disableallsubstitutions = Some(disable);
        self
    }

    common_methods!();
}

/// A text view that renders Markdown.
///
/// See <https://www.alfredapp.com/help/alfredui/components/markdownarea/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MarkdownArea {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    linespacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineheightmultiple: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paragraphspacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    listspacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    autoadjustcontentinsets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urischemes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behaviour: Option<TextBehaviour>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl MarkdownArea {
    /// An empty markdown area.
    pub fn new() -> Self {
        Self::default()
    }

    /// Markdown text to render.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// A file of Markdown to render.
    pub fn src(mut self, path: impl Into<String>) -> Self {
        self.src = Some(path.into());
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// The font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// The text colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// Line spacing within a paragraph.
    pub fn line_spacing(mut self, spacing: f64) -> Self {
        self.linespacing = Some(spacing);
        self
    }

    /// A multiplier for the natural line height.
    pub fn line_height_multiple(mut self, multiple: f64) -> Self {
        self.lineheightmultiple = Some(multiple);
        self
    }

    /// Space between paragraphs.
    pub fn paragraph_spacing(mut self, spacing: f64) -> Self {
        self.paragraphspacing = Some(spacing);
        self
    }

    /// Space between list items.
    pub fn list_spacing(mut self, spacing: f64) -> Self {
        self.listspacing = Some(spacing);
        self
    }

    /// Whether scroll insets adjust for a title bar overlapping the content.
    /// Alfred's default is true.
    pub fn auto_adjust_content_insets(mut self, adjust: bool) -> Self {
        self.autoadjustcontentinsets = Some(adjust);
        self
    }

    /// The URI schemes rendered as links. This replaces Alfred's default of
    /// `http` and `https`, so include them if you still want them.
    pub fn uri_schemes<S: Into<String>>(mut self, schemes: impl IntoIterator<Item = S>) -> Self {
        self.urischemes = Some(schemes.into_iter().map(Into::into).collect());
        self
    }

    /// How text set later combines with the text already there.
    pub fn behaviour(mut self, behaviour: TextBehaviour) -> Self {
        self.behaviour = Some(behaviour);
        self
    }

    common_methods!();
}

/// The style of a [`Progress`] indicator.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStyle {
    /// A bar. This is the default.
    Linear,
    /// A circle.
    Round,
}

/// A progress indicator. Change it with an `update` action on its
/// `progress` property.
///
/// See <https://www.alfredapp.com/help/alfredui/components/progress/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Progress {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<ProgressStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thickness: Option<u32>,
    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Progress {
    /// A linear progress bar at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// The style.
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// The thickness in pixels, 0-32. 0 uses the style's default.
    pub fn thickness(mut self, thickness: u32) -> Self {
        self.thickness = Some(thickness);
        self
    }

    /// The progress from 0.0 to 1.0.
    pub fn progress(mut self, progress: f64) -> Self {
        self.value = Some(progress);
        self
    }

    /// The foreground colour.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    common_methods!();
}

/// Displays an [`Image`].
///
/// See <https://www.alfredapp.com/help/alfredui/components/imageview/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ImageView {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resize: Option<IconResize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contenttintcolour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cornerradius: Option<u32>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl ImageView {
    /// An image view showing `image`.
    pub fn new(image: Image) -> Self {
        Self::empty().image(image)
    }

    /// An image view with no image yet, such as one a dataview row fills in.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The image.
    pub fn image(mut self, image: Image) -> Self {
        self.image = Some(image);
        self
    }

    /// How the image fits the view. Alfred's default is
    /// [`IconResize::Fit`].
    pub fn resize(mut self, resize: IconResize) -> Self {
        self.resize = Some(resize);
        self
    }

    /// The background colour.
    pub fn background(mut self, colour: impl Into<Colour>) -> Self {
        self.background = Some(colour.into());
        self
    }

    /// A tint for content such as SF Symbols.
    pub fn content_tint_colour(mut self, colour: impl Into<Colour>) -> Self {
        self.contenttintcolour = Some(colour.into());
        self
    }

    /// The view's corner radius, 0-32 pixels.
    pub fn corner_radius(mut self, radius: u32) -> Self {
        self.cornerradius = Some(radius);
        self
    }

    common_methods!();
}

/// Web content rendered with WebKit. A URL change fires
/// [`Trigger::Changed`](super::Trigger).
///
/// See <https://www.alfredapp.com/help/alfredui/components/webkitview/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct WebKitView {
    #[serde(flatten)]
    common: Common,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    openlinksexternally: Option<OpenLinks>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum OpenLinks {
    All(bool),
    Matching(String),
}

impl WebKitView {
    /// Loads a URL or file path.
    pub fn url(src: impl Into<String>) -> Self {
        WebKitView {
            src: Some(src.into()),
            ..Self::default()
        }
    }

    /// Renders an HTML string.
    pub fn html(html: impl Into<String>) -> Self {
        WebKitView {
            text: Some(html.into()),
            ..Self::default()
        }
    }

    /// When true, clicked links open in the default browser instead of the
    /// view.
    pub fn open_links_externally(mut self, external: bool) -> Self {
        self.openlinksexternally = Some(OpenLinks::All(external));
        self
    }

    /// Clicked links whose URL matches `regex` open in the default browser.
    pub fn open_links_externally_matching(mut self, regex: impl Into<String>) -> Self {
        self.openlinksexternally = Some(OpenLinks::Matching(regex.into()));
        self
    }

    common_methods!();
}

/// A use of a template from [`Ui::template`](super::Ui::template) or a
/// template file, filling its `{param:name}` placeholders.
///
/// An `id` or `constraints` set here override the template's own.
///
/// See <https://www.alfredapp.com/help/alfredui/templates/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Template {
    #[serde(flatten)]
    common: Common,
    src: String,
    #[serde(skip_serializing_if = "Map::is_empty")]
    params: Map<String, Value>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Template {
    /// Uses the template named `src`, or the template file at that relative
    /// path.
    pub fn new(src: impl Into<String>) -> Self {
        Template {
            common: Common::default(),
            src: src.into(),
            params: Map::new(),
            extra: Map::new(),
        }
    }

    /// Sets the `{param:name}` value.
    pub fn param(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.params.insert(name.into(), value.into());
        self
    }

    common_methods!();
}

/// A component defined in another file, positioned and identified here.
///
/// Unlike a [`Placeholder::file`] placeholder, the `id` and `constraints`
/// stay in this JSON while the file defines the component.
///
/// See <https://www.alfredapp.com/help/alfredui/components/file/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileComponent {
    #[serde(flatten)]
    common: Common,
    src: String,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl FileComponent {
    /// The component in the file at `path`, relative to the `.alfredui`
    /// folder.
    pub fn new(path: impl Into<String>) -> Self {
        FileComponent {
            common: Common::default(),
            src: path.into(),
            extra: Map::new(),
        }
    }

    common_methods!();
}
