//! Layouts and the constraints components give them.

use serde::Serialize;
use serde_json::{Map, Value};

use super::Size;

/// How a [`View`](super::View) arranges its components.
///
/// See <https://www.alfredapp.com/help/alfredui/layouts/>
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// Every component fills the whole view, later ones on top. This is the
    /// default.
    Stack,
    /// Components sit in the top, right, bottom, left or centre region set by
    /// their [`Region`] constraint.
    Border,
    /// An equal-sized grid with `rows` and `cols`, filled row by row.
    Grid,
    /// A horizontal or vertical row of components; see [`Direction`].
    Inline,
    /// Label and field pairs in two aligned columns.
    Form,
    /// A grid with `rows` and `cols` where each component sets its cell and
    /// span with a [`GridCell`] constraint.
    Flex,
}

/// The direction of an inline layout.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Left to right. This is the default.
    Horizontal,
    /// Top to bottom.
    Vertical,
}

/// Layout hints set on a component.
///
/// ```
/// use alfrusco::ui::{Constraints, Insets, Position, Region};
///
/// let constraints = Constraints::new()
///     .region(Region::Bottom)
///     .position(Position::Right)
///     .insets(Insets::all(4));
/// # let _ = constraints;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/layouts/constraints/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Constraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<Region>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stretch: Option<Stretch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grid: Option<GridCell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insets: Option<Insets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<ConstraintSize>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum ConstraintSize {
    Square(u32),
    Size(Size),
}

impl Constraints {
    /// No constraints.
    pub fn new() -> Self {
        Self::default()
    }

    /// The border layout region. Required inside a border layout.
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Where the component sits inside the space the layout gives it.
    pub fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// How a component that isn't positioned to fit stretches.
    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = Some(stretch);
        self
    }

    /// The cell in a flex layout. Required inside a flex layout.
    pub fn grid(mut self, grid: GridCell) -> Self {
        self.grid = Some(grid);
        self
    }

    /// Space around the component, added to its size.
    pub fn insets(mut self, insets: Insets) -> Self {
        self.insets = Some(insets);
        self
    }

    /// Overrides the component's desired size. A zero width or height means
    /// automatic.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(ConstraintSize::Size(size));
        self
    }

    /// Overrides the desired width and height with the same value.
    pub fn square(mut self, size: u32) -> Self {
        self.size = Some(ConstraintSize::Square(size));
        self
    }

    extra_methods!();
}

/// A region of a border layout.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    /// Full width at the top.
    Top,
    /// The right side, between top and bottom.
    Right,
    /// Full width at the bottom.
    Bottom,
    /// The left side, between top and bottom.
    Left,
    /// The remaining space in the middle.
    #[doc(alias = "Center")]
    Centre,
}

/// Where a component sits inside its allotted space.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum Position {
    /// Fill the space. This is the default.
    Fit,
    #[doc(alias = "Center")]
    Centre,
    Top,
    Right,
    Bottom,
    Left,
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
}

/// How a component that isn't positioned to fit stretches.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stretch {
    /// Don't stretch. This is the default.
    None,
    /// Stretch across the available width.
    Horizontal,
    /// Stretch across the available height.
    Vertical,
}

/// A component's cell in a flex layout.
///
/// ```
/// use alfrusco::ui::GridCell;
///
/// // Second column of the first row, two columns wide, taking spare width.
/// let cell = GridCell::at(1, 0).colspan(2).colweight(1.0);
/// # let _ = cell;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct GridCell {
    #[serde(skip_serializing_if = "Option::is_none")]
    col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colspan: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rowspan: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colweight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rowweight: Option<f64>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl GridCell {
    /// The cell at column `col` and row `row`, both counted from 0.
    pub fn at(col: u32, row: u32) -> Self {
        GridCell {
            col: Some(col),
            row: Some(row),
            ..Self::default()
        }
    }

    /// The column, counted from 0.
    pub fn col(mut self, col: u32) -> Self {
        self.col = Some(col);
        self
    }

    /// The row, counted from 0.
    pub fn row(mut self, row: u32) -> Self {
        self.row = Some(row);
        self
    }

    /// How many columns the component spans.
    pub fn colspan(mut self, colspan: u32) -> Self {
        self.colspan = Some(colspan);
        self
    }

    /// How many rows the component spans.
    pub fn rowspan(mut self, rowspan: u32) -> Self {
        self.rowspan = Some(rowspan);
        self
    }

    /// The column's share of extra width on resize, 0.0 to 1.0.
    pub fn colweight(mut self, weight: f64) -> Self {
        self.colweight = Some(weight);
        self
    }

    /// The row's share of extra height on resize, 0.0 to 1.0.
    pub fn rowweight(mut self, weight: f64) -> Self {
        self.rowweight = Some(weight);
        self
    }

    extra_methods!();
}

/// Space in pixels around a component.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Insets {
    #[serde(skip_serializing_if = "Option::is_none")]
    top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    right: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bottom: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    left: Option<u32>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Insets {
    /// Every side set separately, in CSS order.
    pub fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self::default()
            .top(top)
            .right(right)
            .bottom(bottom)
            .left(left)
    }

    /// The same inset on every side.
    pub fn all(inset: u32) -> Self {
        Self::new(inset, inset, inset, inset)
    }

    /// The top inset.
    pub fn top(mut self, inset: u32) -> Self {
        self.top = Some(inset);
        self
    }

    /// The right inset.
    pub fn right(mut self, inset: u32) -> Self {
        self.right = Some(inset);
        self
    }

    /// The bottom inset.
    pub fn bottom(mut self, inset: u32) -> Self {
        self.bottom = Some(inset);
        self
    }

    /// The left inset.
    pub fn left(mut self, inset: u32) -> Self {
        self.left = Some(inset);
        self
    }

    extra_methods!();
}
