//! The top-level window.

use serde::Serialize;
use serde_json::{Map, Value};

use super::{Component, Size};

/// The window that shows a UI. Most settings only apply when the UI runs
/// outside Alfred's own window.
///
/// ```
/// use alfrusco::ui::{Label, Size, Toolbar, Window};
///
/// let window = Window::new(Label::new("Hello"))
///     .title("Greeter")
///     .size(Size::new(480, 320))
///     .toolbar(Toolbar::unified())
///     .padding(16);
/// # let _ = window;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/window/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Window {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minsize: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxsize: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cornerradius: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blur: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    movablebybackground: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    borderless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closeable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimisable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resizeable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fullscreenable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fullsizecontentview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frameautosave: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    titlevisibility: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    titlebarappearstransparent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    toolbar: Option<Toolbar>,
    root: Box<Component>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Window {
    /// A window showing `root`, usually a [`View`](super::View).
    pub fn new(root: impl Into<Component>) -> Self {
        Window {
            id: None,
            title: None,
            size: None,
            minsize: None,
            maxsize: None,
            padding: None,
            cornerradius: None,
            blur: None,
            movablebybackground: None,
            level: None,
            appearance: None,
            borderless: None,
            closeable: None,
            minimisable: None,
            resizeable: None,
            fullscreenable: None,
            fullsizecontentview: None,
            frameautosave: None,
            titlevisibility: None,
            titlebarappearstransparent: None,
            toolbar: None,
            root: Box::new(root.into()),
            extra: Map::new(),
        }
    }

    /// The window's id, for actions that target it.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The size, 50-10000 pixels each way. Outside Alfred the default is
    /// 800x600; inside, the width follows the theme and the height fits the UI.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    /// The minimum size.
    pub fn min_size(mut self, size: Size) -> Self {
        self.minsize = Some(size);
        self
    }

    /// The maximum size.
    pub fn max_size(mut self, size: Size) -> Self {
        self.maxsize = Some(size);
        self
    }

    /// Space around the UI inside the window, 0-32 pixels.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// The corner radius of a borderless window, 0-128 pixels.
    pub fn corner_radius(mut self, radius: u32) -> Self {
        self.cornerradius = Some(radius);
        self
    }

    /// Whether the window background is blurred. Alfred's default is true.
    pub fn blur(mut self, blur: bool) -> Self {
        self.blur = Some(blur);
        self
    }

    /// Whether dragging the background moves the window.
    pub fn movable_by_background(mut self, movable: bool) -> Self {
        self.movablebybackground = Some(movable);
        self
    }

    /// The AppKit window level. Use with care.
    pub fn level(mut self, level: i64) -> Self {
        self.level = Some(level);
        self
    }

    /// Forces light or dark mode. The default is
    /// [`Appearance::Automatic`].
    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// A window with no title bar, like Alfred's own. Disables the toolbar.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = Some(borderless);
        self
    }

    /// Whether the close button works. Alfred's default is true.
    pub fn closeable(mut self, closeable: bool) -> Self {
        self.closeable = Some(closeable);
        self
    }

    /// Whether the minimise button works. Alfred's default is true.
    pub fn minimisable(mut self, minimisable: bool) -> Self {
        self.minimisable = Some(minimisable);
        self
    }

    /// Whether the window can be resized. Alfred's default is true.
    pub fn resizeable(mut self, resizeable: bool) -> Self {
        self.resizeable = Some(resizeable);
        self
    }

    /// Whether the maximise button goes full screen. Alfred's default is true.
    pub fn fullscreenable(mut self, fullscreenable: bool) -> Self {
        self.fullscreenable = Some(fullscreenable);
        self
    }

    /// Extends the content up behind the toolbar.
    pub fn full_size_content_view(mut self, full_size: bool) -> Self {
        self.fullsizecontentview = Some(full_size);
        self
    }

    /// Remembers the window frame between runs. Only for UIs run outside
    /// Alfred.
    pub fn frame_autosave(mut self, autosave: bool) -> Self {
        self.frameautosave = Some(autosave);
        self
    }

    /// Whether the title is shown. Alfred's default is true.
    pub fn title_visibility(mut self, visible: bool) -> Self {
        self.titlevisibility = Some(visible);
        self
    }

    /// Makes the title bar or toolbar transparent.
    pub fn titlebar_appears_transparent(mut self, transparent: bool) -> Self {
        self.titlebarappearstransparent = Some(transparent);
        self
    }

    /// Adds a toolbar.
    pub fn toolbar(mut self, toolbar: Toolbar) -> Self {
        self.toolbar = Some(toolbar);
        self
    }

    extra_methods!();
}

/// A light or dark appearance override.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    /// Follow macOS. The window's default.
    Automatic,
    /// Follow the parent view. A view's default.
    Inherit,
    /// Always light.
    Light,
    /// Always dark.
    Dark,
}

/// A window toolbar.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Toolbar {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    kind: Option<ToolbarType>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// The style of a [`Toolbar`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolbarType {
    /// The unified toolbar. This is the default.
    Unified,
    /// The compact unified toolbar.
    UnifiedCompact,
}

impl Toolbar {
    /// A toolbar of `kind`.
    pub fn new(kind: ToolbarType) -> Self {
        Toolbar {
            kind: Some(kind),
            extra: Map::new(),
        }
    }

    /// The unified toolbar.
    pub fn unified() -> Self {
        Self::new(ToolbarType::Unified)
    }

    /// The compact unified toolbar.
    pub fn unified_compact() -> Self {
        Self::new(ToolbarType::UnifiedCompact)
    }

    extra_methods!();
}
