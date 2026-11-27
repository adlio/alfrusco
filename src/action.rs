use serde::Serialize;

use crate::Arg;

/// How Alfred interprets an item's `arg` (the item's `type` key).
///
/// See <https://www.alfredapp.com/help/workflows/inputs/script-filter/json/>
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ItemType {
    /// Treat the arg as a plain string. This is Alfred's default.
    #[serde(rename = "default")]
    Default,
    /// Treat the arg as a file path, enabling Universal Actions, the File
    /// Buffer and Quick Look. Alfred skips paths that don't exist.
    #[serde(rename = "file")]
    File,
    /// Like [`ItemType::File`], but Alfred doesn't check that the path
    /// exists. Use only when you know it does and need the speed.
    #[serde(rename = "file:skipcheck")]
    FileSkipCheck,
}

/// The content Alfred hands to Universal Actions when the user opens them on
/// an item, with each value tagged by content type.
///
/// Set it with [`Item::universal_action`](crate::Item::universal_action). For
/// content whose type Alfred should infer, use
/// [`Item::action`](crate::Item::action) instead.
///
/// ```
/// use alfrusco::{Item, UniversalAction};
///
/// let item = Item::new("Alfred").universal_action(
///     UniversalAction::new()
///         .text(["one", "two"])
///         .url("https://www.alfredapp.com")
///         .file("~/Desktop"),
/// );
/// ```
///
/// See <https://www.alfredapp.com/help/workflows/inputs/script-filter/json/>
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
pub struct UniversalAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Arg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Arg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<Arg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto: Option<Arg>,
}

impl UniversalAction {
    /// An action with no content. Add values with [`text`](Self::text),
    /// [`url`](Self::url), [`file`](Self::file) or [`auto`](Self::auto).
    pub fn new() -> Self {
        Self::default()
    }

    /// One or more values offered as text.
    pub fn text(mut self, value: impl Into<Arg>) -> Self {
        self.text = Some(value.into());
        self
    }

    /// One or more values offered as URLs.
    pub fn url(mut self, value: impl Into<Arg>) -> Self {
        self.url = Some(value.into());
        self
    }

    /// One or more values offered as file paths.
    pub fn file(mut self, value: impl Into<Arg>) -> Self {
        self.file = Some(value.into());
        self
    }

    /// One or more values whose type Alfred infers (file, URL or text).
    pub fn auto(mut self, value: impl Into<Arg>) -> Self {
        self.auto = Some(value.into());
        self
    }
}

/// The serialized form of an item's `action` key: either bare values whose
/// type Alfred infers, or a typed object.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub(crate) enum ItemAction {
    Inferred(Arg),
    Typed(UniversalAction),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_item_type_values() {
        assert_eq!(
            serde_json::to_value(ItemType::Default).unwrap(),
            json!("default")
        );
        assert_eq!(serde_json::to_value(ItemType::File).unwrap(), json!("file"));
        assert_eq!(
            serde_json::to_value(ItemType::FileSkipCheck).unwrap(),
            json!("file:skipcheck")
        );
    }

    #[test]
    fn test_universal_action_object() {
        let action = UniversalAction::new()
            .text(["one", "two", "three"])
            .url("https://www.alfredapp.com")
            .file("~/Desktop")
            .auto("~/Pictures");
        assert_eq!(
            serde_json::to_value(ItemAction::Typed(action)).unwrap(),
            json!({
                "text": ["one", "two", "three"],
                "url": "https://www.alfredapp.com",
                "file": "~/Desktop",
                "auto": "~/Pictures"
            })
        );
    }

    #[test]
    fn test_universal_action_omits_unset_types() {
        let action = UniversalAction::new().url("https://example.com");
        assert_eq!(
            serde_json::to_value(action).unwrap(),
            json!({"url": "https://example.com"})
        );
    }

    #[test]
    fn test_inferred_action() {
        assert_eq!(
            serde_json::to_value(ItemAction::Inferred("Alfred is Great".into())).unwrap(),
            json!("Alfred is Great")
        );
        assert_eq!(
            serde_json::to_value(ItemAction::Inferred(vec!["a", "b"].into())).unwrap(),
            json!(["a", "b"])
        );
    }
}
