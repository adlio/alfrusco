//! Property types shared across the UI: colours, fonts, images, sizes, menus
//! and shortcuts.

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

use super::{Action, Constraints, Event};

/// Serializes a one-element list as the element itself and anything longer as
/// an array, matching the `X | ARRAY` properties in Alfred UI.
pub(crate) fn one_or_many<T: Serialize, S: Serializer>(
    values: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match values {
        [one] => one.serialize(serializer),
        many => many.serialize(serializer),
    }
}

/// A colour value: hex, `rgb()`, `hsv()`, a macOS system colour name, a
/// colour constant from [`Ui::colour`](super::Ui::colour), or a pair of colours
/// for light and dark mode.
///
/// ```
/// use alfrusco::ui::Colour;
///
/// let accent = Colour::named("systemOrangeColor");
/// let swatch = Colour::hex("#ffe135");
/// let shade = Colour::rgba(0, 0, 255, 0.3);
/// let adaptive = Colour::light_dark(Colour::hex("#222222"), Colour::hex("#eeeeee"));
/// assert_eq!(serde_json::to_value(&swatch).unwrap(), "#ffe135");
/// assert_eq!(
///     serde_json::to_value(&adaptive).unwrap(),
///     serde_json::json!({ "light": "#222222", "dark": "#eeeeee" })
/// );
/// # let _ = (accent, shade);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/properties/colour/>
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Colour(ColourValue);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum ColourValue {
    Text(String),
    Adaptive {
        light: Box<Colour>,
        dark: Box<Colour>,
    },
    Null,
}

impl Colour {
    /// Any colour string Alfred accepts, written as is.
    pub fn new(value: impl Into<String>) -> Self {
        Colour(ColourValue::Text(value.into()))
    }

    /// A macOS system colour such as `"systemOrangeColor"` or
    /// `"secondaryLabelColor"`, or a colour constant defined in the UI.
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(name)
    }

    /// A hex colour, `#RRGGBB` or `#RRGGBBAA`. A missing `#` is added.
    pub fn hex(hex: impl Into<String>) -> Self {
        let hex = hex.into();
        if hex.starts_with('#') {
            Self::new(hex)
        } else {
            Self::new(format!("#{hex}"))
        }
    }

    /// `rgb(red, green, blue)`.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new(format!("rgb({red}, {green}, {blue})"))
    }

    /// `rgb(red, green, blue, alpha)` with alpha from 0.0 to 1.0.
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: f64) -> Self {
        Self::new(format!("rgb({red}, {green}, {blue}, {alpha})"))
    }

    /// `hsv(degrees, saturation, brightness)`: hue 0-359, the others 0-100.
    pub fn hsv(hue: u16, saturation: u8, brightness: u8) -> Self {
        Self::new(format!("hsv({hue}, {saturation}, {brightness})"))
    }

    /// `hsv(degrees, saturation, brightness, alpha)` with alpha from 0.0 to 1.0.
    pub fn hsva(hue: u16, saturation: u8, brightness: u8, alpha: f64) -> Self {
        Self::new(format!("hsv({hue}, {saturation}, {brightness}, {alpha})"))
    }

    /// The text colour of the selected result in the user's Alfred theme.
    pub fn alfred_selected_result_text() -> Self {
        Self::new("alfredSelectedResultTextColor")
    }

    /// The background colour of the selected result in the user's Alfred theme.
    pub fn alfred_selected_result_background() -> Self {
        Self::new("alfredSelectedResultBackgroundColor")
    }

    /// One colour in light mode and another in dark mode.
    pub fn light_dark(light: impl Into<Colour>, dark: impl Into<Colour>) -> Self {
        Colour(ColourValue::Adaptive {
            light: Box::new(light.into()),
            dark: Box::new(dark.into()),
        })
    }

    /// JSON `null`: removes a background or border, or restores the default
    /// text or tint colour. Mostly useful in dataview cell `init` and in
    /// `update` actions.
    pub fn none() -> Self {
        Colour(ColourValue::Null)
    }
}

impl From<&str> for Colour {
    fn from(value: &str) -> Self {
        Colour::new(value)
    }
}

impl From<String> for Colour {
    fn from(value: String) -> Self {
        Colour::new(value)
    }
}

impl From<Colour> for Value {
    fn from(colour: Colour) -> Self {
        match colour.0 {
            ColourValue::Text(text) => Value::String(text),
            ColourValue::Adaptive { light, dark } => {
                let mut map = Map::new();
                map.insert("light".into(), (*light).into());
                map.insert("dark".into(), (*dark).into());
                Value::Object(map)
            }
            ColourValue::Null => Value::Null,
        }
    }
}

/// Font settings for text components.
///
/// ```
/// use alfrusco::ui::{Font, FontSize, FontWeight};
///
/// let font = Font::new().size(FontSize::Large).weight(FontWeight::Bold);
/// let fixed = Font::new().pixels(14).monospaced_digits();
/// # let _ = (font, fixed);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/properties/font/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Font {
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<FontSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    monospaced: Option<Monospaced>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<FontWeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<FontWidth>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(untagged)]
enum Monospaced {
    Flag(bool),
    Digits(&'static str),
}

impl Font {
    /// A font with every property at Alfred's default.
    pub fn new() -> Self {
        Self::default()
    }

    /// One of the named text sizes.
    pub fn size(mut self, size: FontSize) -> Self {
        self.size = Some(size);
        self
    }

    /// A text size in pixels.
    pub fn pixels(mut self, pixels: u32) -> Self {
        self.size = Some(FontSize::Pixels(pixels));
        self
    }

    /// When true, use the system monospace font.
    pub fn monospaced(mut self, monospaced: bool) -> Self {
        self.monospaced = Some(Monospaced::Flag(monospaced));
        self
    }

    /// Use the system font with fixed-width digits.
    pub fn monospaced_digits(mut self) -> Self {
        self.monospaced = Some(Monospaced::Digits("digits"));
        self
    }

    /// The font weight.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// The font width. Ignored for monospaced fonts.
    pub fn width(mut self, width: FontWidth) -> Self {
        self.width = Some(width);
        self
    }

    extra_methods!();
}

/// A font size: a size in pixels or one of the system sizes.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSize {
    /// A size in pixels.
    Pixels(u32),
    /// The system default size.
    Default,
    /// Hairline.
    Hairline,
    /// Extra small.
    ExtraSmall,
    /// Small.
    Small,
    /// Medium.
    Medium,
    /// Large.
    Large,
    /// Extra large.
    ExtraLarge,
    /// Ginormous.
    Ginormous,
}

impl Serialize for FontSize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let name = match self {
            FontSize::Pixels(pixels) => return serializer.serialize_u32(*pixels),
            FontSize::Default => "default",
            FontSize::Hairline => "hairline",
            FontSize::ExtraSmall => "extrasmall",
            FontSize::Small => "small",
            FontSize::Medium => "medium",
            FontSize::Large => "large",
            FontSize::ExtraLarge => "extralarge",
            FontSize::Ginormous => "ginormous",
        };
        serializer.serialize_str(name)
    }
}

/// Font weight, matching the AppKit `NSFontWeight` constants.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum FontWeight {
    UltraLight,
    Thin,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    Heavy,
    Black,
}

/// Font width, matching the AppKit `NSFontWidth` constants.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum FontWidth {
    Compressed,
    Condensed,
    Standard,
    Expanded,
}

/// An image: a file, a file's Finder icon, a file type's icon or an SF Symbol.
///
/// ```
/// use alfrusco::ui::{Image, Size};
///
/// let logo = Image::file("resources/logo.png");
/// let smile = Image::symbol("face.smiling").size(Size::with_width(32));
/// # let _ = (logo, smile);
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/properties/image/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Image {
    #[serde(rename = "type")]
    kind: ImageType,
    src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<Size>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// What an [`Image`]'s `src` refers to.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    /// An image file, relative to the `.alfredui` folder or absolute.
    File,
    /// The Finder icon of the file at the path.
    FileIcon,
    /// The icon of the file's type.
    FileTypeIcon,
    /// An SF Symbol name.
    Symbol,
}

impl Image {
    /// An image of `kind` loaded from `src`.
    pub fn new(kind: ImageType, src: impl Into<String>) -> Self {
        Image {
            kind,
            src: src.into(),
            size: None,
            extra: Map::new(),
        }
    }

    /// An image file, relative to the `.alfredui` folder or absolute.
    pub fn file(path: impl Into<String>) -> Self {
        Self::new(ImageType::File, path)
    }

    /// The Finder icon of the file or folder at `path`.
    pub fn file_icon(path: impl Into<String>) -> Self {
        Self::new(ImageType::FileIcon, path)
    }

    /// The icon of the type of the file at `path`.
    pub fn file_type_icon(path: impl Into<String>) -> Self {
        Self::new(ImageType::FileTypeIcon, path)
    }

    /// An SF Symbol, such as `"arrow.clockwise"`.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::new(ImageType::Symbol, name)
    }

    /// The preferred size. Set only a width or height to keep the aspect ratio.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    extra_methods!();
}

/// A width and height in pixels. Either may be left unset.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Size {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Size {
    /// Both a width and a height.
    pub fn new(width: u32, height: u32) -> Self {
        Self::with_width(width).height(height)
    }

    /// Only a width.
    pub fn with_width(width: u32) -> Self {
        Self::default().width(width)
    }

    /// Only a height.
    pub fn with_height(height: u32) -> Self {
        Self::default().height(height)
    }

    /// Sets the width.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    extra_methods!();
}

/// Horizontal and vertical space around the text inside a text component.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TextInsets {
    #[serde(skip_serializing_if = "Option::is_none")]
    horizontal: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vertical: Option<u32>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl TextInsets {
    /// Both insets, in pixels.
    pub fn new(horizontal: u32, vertical: u32) -> Self {
        Self::default().horizontal(horizontal).vertical(vertical)
    }

    /// The inset left and right of the text.
    pub fn horizontal(mut self, horizontal: u32) -> Self {
        self.horizontal = Some(horizontal);
        self
    }

    /// The inset above and below the text.
    pub fn vertical(mut self, vertical: u32) -> Self {
        self.vertical = Some(vertical);
        self
    }

    extra_methods!();
}

/// One entry in a right-click or popup menu.
///
/// See <https://www.alfredapp.com/help/alfredui/properties/menu/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MenuItem {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    kind: Option<MenuItemType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<MenuItemState>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    submenu: Vec<MenuItem>,
    #[serde(serialize_with = "one_or_many", skip_serializing_if = "Vec::is_empty")]
    action: Vec<Action>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// The kind of a [`MenuItem`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MenuItemType {
    /// A text item. This is the default.
    Text,
    /// A separator line.
    Separator,
}

/// The tick state shown beside a [`MenuItem`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MenuItemState {
    /// No mark. This is the default.
    Off,
    /// A tick.
    On,
    /// A dash.
    Mixed,
}

impl MenuItem {
    /// A text item.
    pub fn new(text: impl Into<String>) -> Self {
        MenuItem {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    /// A separator line.
    pub fn separator() -> Self {
        MenuItem {
            kind: Some(MenuItemType::Separator),
            ..Self::default()
        }
    }

    /// The tick state.
    pub fn state(mut self, state: MenuItemState) -> Self {
        self.state = Some(state);
        self
    }

    /// Adds an action to run when the item is chosen.
    pub fn action(mut self, action: impl Into<Action>) -> Self {
        self.action.push(action.into());
        self
    }

    /// Adds an item to this item's submenu. Alfred ignores the item's own
    /// action once it has a submenu.
    pub fn submenu_item(mut self, item: MenuItem) -> Self {
        self.submenu.push(item);
        self
    }

    extra_methods!();
}

/// A key combination on a view that runs actions while focus is inside it.
///
/// See <https://www.alfredapp.com/help/alfredui/properties/shortcut/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shortcut {
    #[serde(rename = "shortcut")]
    keys: String,
    #[serde(serialize_with = "one_or_many", skip_serializing_if = "Vec::is_empty")]
    action: Vec<Action>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Shortcut {
    /// A shortcut such as `"⌘L"` or `"↓"` that runs `action`.
    pub fn new(shortcut: impl Into<String>, action: impl Into<Action>) -> Self {
        Shortcut {
            keys: shortcut.into(),
            action: vec![action.into()],
            extra: Map::new(),
        }
    }

    /// Adds another action to run with the first.
    pub fn action(mut self, action: impl Into<Action>) -> Self {
        self.action.push(action.into());
        self
    }

    extra_methods!();
}

/// VoiceOver text for a component.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Accessibility {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Accessibility {
    /// A short label, ideally one capitalised word with no full stop.
    pub fn label(label: impl Into<String>) -> Self {
        Accessibility {
            label: Some(label.into()),
            ..Self::default()
        }
    }

    /// A sentence explaining what activating the component does.
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    extra_methods!();
}

/// The properties every component shares.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct Common {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) constraints: Option<Constraints>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) events: Vec<Event>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) focus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tooltip: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) menu: Vec<MenuItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) shortcut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) accessibility: Option<Accessibility>,
}
