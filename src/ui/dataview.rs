//! The dataview component: a list or grid of cells filled from data rows.

use serde::Serialize;
use serde_json::{Map, Value};

use super::property::Common;
use super::{Component, Properties};
use crate::Arg;

/// A list or grid of cells. Each data [`Row`] sets properties on the
/// components of one [`Cell`], keyed `"componentid.property"`.
///
/// ```
/// use alfrusco::ui::{Cell, Data, DataView, Filter, Label, Placeholder, Row};
///
/// let fruit = DataView::new(
///     Cell::new(Label::empty().id("name")).height(32),
///     Data::rows([
///         Row::new().value("name", "text", "Banana"),
///         Row::new().value("name", "text", "Cherry"),
///     ])
///     .filter(Filter::new("name.text", Placeholder::value("search", "text"))),
/// )
/// .id("fruit");
/// # let _ = fruit;
/// ```
///
/// See <https://www.alfredapp.com/help/alfredui/components/dataview/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DataView {
    #[serde(flatten)]
    common: Common,
    cell: Cell,
    data: Data,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cols: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    padding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    autoadjustcontentinsets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behaviour: Option<DataViewBehaviour>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl DataView {
    /// A dataview that renders each entry of `data` with `cell`.
    pub fn new(cell: Cell, data: Data) -> Self {
        DataView {
            common: Common::default(),
            cell,
            data,
            placeholder: None,
            cols: None,
            padding: None,
            spacing: None,
            autoadjustcontentinsets: None,
            behaviour: None,
            extra: Map::new(),
        }
    }

    /// Text shown when there's no data.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Columns, 1-16. More than 1 lays cells out row by row as a grid.
    pub fn cols(mut self, cols: u32) -> Self {
        self.cols = Some(cols);
        self
    }

    /// Space inside the dataview's edges, 0-64 pixels.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Space between cells, 0-64 pixels.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Whether scroll insets adjust for a title bar overlapping the content.
    /// Alfred's default is true.
    pub fn auto_adjust_content_insets(mut self, adjust: bool) -> Self {
        self.autoadjustcontentinsets = Some(adjust);
        self
    }

    /// Selection behaviour.
    pub fn behaviour(mut self, behaviour: DataViewBehaviour) -> Self {
        self.behaviour = Some(behaviour);
        self
    }

    common_methods!();
}

/// The UI each data entry is rendered into. Cells are reused, so `init`
/// resets the properties rows change, and `selected` styles the selected
/// cell.
///
/// ```
/// use alfrusco::ui::{Cell, Colour, Label, View};
///
/// let cell = Cell::new(View::vertical().id("row").padding(8).child(Label::empty().id("name")))
///     .height(40)
///     .init("row", "background", Colour::none())
///     .selected("row", "background", Colour::alfred_selected_result_background());
/// # let _ = cell;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cell {
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    root: Box<Component>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    init: Properties,
    #[serde(skip_serializing_if = "Map::is_empty")]
    selected: Properties,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Cell {
    /// A cell whose content is `root`.
    pub fn new(root: impl Into<Component>) -> Self {
        Cell {
            height: None,
            root: Box::new(root.into()),
            init: Map::new(),
            selected: Map::new(),
            extra: Map::new(),
        }
    }

    /// The height of every cell, 12-1024 pixels. Alfred's default is 64.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Resets `property` of component `id` before each row is applied.
    pub fn init(
        mut self,
        id: impl AsRef<str>,
        property: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> Self {
        self.init.insert(key(id, property), value.into());
        self
    }

    /// Sets `property` of component `id` on the selected cell.
    pub fn selected(
        mut self,
        id: impl AsRef<str>,
        property: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> Self {
        self.selected.insert(key(id, property), value.into());
        self
    }

    extra_methods!();
}

fn key(id: impl AsRef<str>, property: impl AsRef<str>) -> String {
    format!("{}.{}", id.as_ref(), property.as_ref())
}

/// One data entry for a dataview: the values to set on the cell's
/// components, plus an optional `uid` used for selection, filtering and
/// later edits.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Row {
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Row {
    /// An empty row.
    pub fn new() -> Self {
        Self::default()
    }

    /// The row's uid.
    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    /// Sets `property` of the cell component with id `id`, written as the key
    /// `"id.property"`.
    pub fn value(
        mut self,
        id: impl AsRef<str>,
        property: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> Self {
        self.extra.insert(key(id, property), value.into());
        self
    }

    extra_methods!();
}

/// Where a dataview's data comes from: Alfred's `basic` provider.
///
/// See <https://www.alfredapp.com/help/alfredui/components/dataview/providers/basic/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Data {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<DataSrc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modify: Option<Modify>,
    #[serde(skip_serializing_if = "Option::is_none")]
    select: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum DataSrc {
    Rows(Vec<Row>),
    Placeholder(String),
}

impl Data {
    fn with_src(src: Option<DataSrc>) -> Self {
        Data {
            kind: "basic",
            src,
            filter: None,
            modify: None,
            select: None,
            extra: Map::new(),
        }
    }

    /// Data given inline.
    pub fn rows(rows: impl IntoIterator<Item = Row>) -> Self {
        Self::with_src(Some(DataSrc::Rows(rows.into_iter().collect())))
    }

    /// Data from a placeholder, such as
    /// [`Placeholder::file_raw`](super::Placeholder::file_raw) for a large
    /// JSON file.
    pub fn source(placeholder: impl Into<String>) -> Self {
        Self::with_src(Some(DataSrc::Placeholder(placeholder.into())))
    }

    /// No `src`, for an `update` action that only changes the filter, edits
    /// rows with [`Data::modify`] or moves the selection.
    pub fn change() -> Self {
        Self::with_src(None)
    }

    /// Filters the rows by user input.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Inserts, replaces or deletes rows of the data already shown.
    pub fn modify(mut self, modify: Modify) -> Self {
        self.modify = Some(modify);
        self
    }

    /// Selects the row with this uid. An empty or unknown uid resets the
    /// selection.
    pub fn select(mut self, uid: impl Into<String>) -> Self {
        self.select = Some(uid.into());
        self
    }

    extra_methods!();
}

impl From<Data> for Value {
    fn from(data: Data) -> Self {
        serde_json::to_value(data).unwrap_or(Value::Null)
    }
}

/// Alfred's `basic` word-match filter.
///
/// See <https://www.alfredapp.com/help/alfredui/components/dataview/filters/basic/>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Filter {
    #[serde(rename = "type")]
    kind: &'static str,
    args: Arg,
    #[serde(rename = "match")]
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    matchmode: Option<MatchMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matchempty: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Filter {
    /// Matches `query`, usually a `{value:field.text}` placeholder, against
    /// the row keys in `fields`, such as `"name.text"`.
    pub fn new(fields: impl Into<Arg>, query: impl Into<String>) -> Self {
        Filter {
            kind: "basic",
            args: fields.into(),
            query: query.into(),
            matchmode: None,
            matchempty: None,
            extra: Map::new(),
        }
    }

    /// How the query matches.
    pub fn match_mode(mut self, mode: MatchMode) -> Self {
        self.matchmode = Some(mode);
        self
    }

    /// When false, an empty query shows no rows. Alfred's default is true.
    pub fn match_empty(mut self, show_all: bool) -> Self {
        self.matchempty = Some(show_all);
        self
    }

    extra_methods!();
}

/// How a [`Filter`] matches the query.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    /// An exact match from the start of any word. This is the default.
    ExactFromStartOrWhitespace,
    /// An exact match from the start of the phrase.
    ExactFromStart,
    /// Every query word matches, in any order.
    WordMatchAnyOrder,
    /// Every query word matches, in order.
    WordMatchSequential,
}

/// Edits to the rows a dataview already shows.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Modify {
    #[serde(skip_serializing_if = "Option::is_none")]
    insert: Option<InsertRows>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<ReplaceRows>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<DeleteRows>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Modify {
    /// No edits yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts rows.
    pub fn insert(mut self, insert: InsertRows) -> Self {
        self.insert = Some(insert);
        self
    }

    /// Replaces rows by uid.
    pub fn replace(mut self, replace: ReplaceRows) -> Self {
        self.replace = Some(replace);
        self
    }

    /// Deletes rows by uid.
    pub fn delete(mut self, delete: DeleteRows) -> Self {
        self.delete = Some(delete);
        self
    }

    extra_methods!();
}

/// Where [`InsertRows`] puts the new rows relative to a uid.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertPosition {
    /// After the uid, or at the end. This is the default.
    After,
    /// Before the uid, or at the start.
    Before,
}

/// Rows to insert into a dataview's data.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InsertRows {
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<InsertPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    autoselect: Option<bool>,
    src: Vec<Row>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl InsertRows {
    /// Inserts `rows` at the end.
    pub fn new(rows: impl IntoIterator<Item = Row>) -> Self {
        InsertRows {
            position: None,
            uid: None,
            autoselect: None,
            src: rows.into_iter().collect(),
            extra: Map::new(),
        }
    }

    /// Inserts after the row with this uid.
    pub fn after(mut self, uid: impl Into<String>) -> Self {
        self.position = Some(InsertPosition::After);
        self.uid = Some(uid.into());
        self
    }

    /// Inserts before the row with this uid.
    pub fn before(mut self, uid: impl Into<String>) -> Self {
        self.position = Some(InsertPosition::Before);
        self.uid = Some(uid.into());
        self
    }

    /// Inserts at the start or end, depending on `position`.
    pub fn position(mut self, position: InsertPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Whether the first inserted row is selected. Alfred's default is true.
    pub fn autoselect(mut self, autoselect: bool) -> Self {
        self.autoselect = Some(autoselect);
        self
    }

    extra_methods!();
}

/// Rows to replace by uid. Unknown uids are ignored.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ReplaceRows {
    #[serde(skip_serializing_if = "Option::is_none")]
    autoselect: Option<bool>,
    srcmap: std::collections::BTreeMap<String, Row>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl ReplaceRows {
    /// Nothing to replace yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the row with uid `uid` by `row`.
    pub fn row(mut self, uid: impl Into<String>, row: Row) -> Self {
        self.srcmap.insert(uid.into(), row);
        self
    }

    /// Whether the first replaced row is selected. Alfred's default is true.
    pub fn autoselect(mut self, autoselect: bool) -> Self {
        self.autoselect = Some(autoselect);
        self
    }

    extra_methods!();
}

/// Rows to delete by uid.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeleteRows {
    #[serde(skip_serializing_if = "Option::is_none")]
    autoselect: Option<bool>,
    uids: Vec<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl DeleteRows {
    /// Deletes the rows with these uids.
    pub fn new<S: Into<String>>(uids: impl IntoIterator<Item = S>) -> Self {
        DeleteRows {
            autoselect: None,
            uids: uids.into_iter().map(Into::into).collect(),
            extra: Map::new(),
        }
    }

    /// Whether the cell at the deleted index is selected afterwards. Alfred's
    /// default is true.
    pub fn autoselect(mut self, autoselect: bool) -> Self {
        self.autoselect = Some(autoselect);
        self
    }

    extra_methods!();
}

/// A dataview's `behaviour` settings.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DataViewBehaviour {
    #[serde(skip_serializing_if = "Option::is_none")]
    selection: Option<Selection>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl DataViewBehaviour {
    /// Alfred's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selection behaviour.
    pub fn selection(mut self, selection: Selection) -> Self {
        self.selection = Some(selection);
        self
    }

    extra_methods!();
}

/// How a dataview's selection behaves.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Selection {
    #[serde(skip_serializing_if = "Option::is_none")]
    wrap: Option<SelectionWrap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowempty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hover: Option<bool>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// Whether moving past the last row wraps to the first.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionWrap {
    /// No wrapping. This is the default.
    None,
    /// Wrap back to the start.
    Vertical,
}

impl Selection {
    /// Alfred's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selection wrapping.
    pub fn wrap(mut self, wrap: SelectionWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Allows no cell to be selected. The changed event then passes `null`.
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allowempty = Some(allow);
        self
    }

    /// Selects the cell under the pointer. Each change fires a changed event.
    pub fn hover(mut self, hover: bool) -> Self {
        self.hover = Some(hover);
        self
    }

    extra_methods!();
}
