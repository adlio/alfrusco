use std::collections::HashMap;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Serialize;

mod arg;
pub mod icon;
mod modifiers;
mod text;

pub use arg::Arg;
pub use icon::Icon;
pub use modifiers::{Key, Modifier};
pub use text::Text;

pub fn filter_and_sort_items(items: Vec<Item>, query: String) -> Vec<Item> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_items: Vec<(Item, i64)> = items
        .into_iter()
        .filter_map(|item| {
            let subtitle = item.subtitle.as_deref().unwrap_or_default();
            let combined = format!("{} : {}", subtitle, item.title);
            matcher
                .fuzzy_match(&combined, &query)
                .map(|score| (item, score))
        })
        .collect();

    // Sort by score in descending order
    filtered_items.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    filtered_items.into_iter().map(|(item, _)| item).collect()
}

/// Item represents a single choice in the Alfred selection UI.
///
/// The fields here are designed around the Script Filter JSON format defined
/// on the Alfred web site:
///
/// (https://www.alfredapp.com/help/workflows/inputs/script-filter/json/).
///
/// Fields here include all current features, but the struct is marked
/// non-exhaustive to allow for future expansion of the Alfred JSON format.
/// Builder functions are provided for each field to allow for easy
/// specification of each field.
///
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Item {
    pub(crate) title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) uid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) arg: Option<Arg>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub(crate) variables: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) icon: Option<Icon>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) valid: Option<bool>,

    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub(crate) r#match: Option<String>,

    #[serde(rename = "mods", skip_serializing_if = "HashMap::is_empty")]
    pub(crate) modifiers: HashMap<String, Modifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) autocomplete: Option<String>,

    #[serde(rename = "quicklookurl", skip_serializing_if = "Option::is_none")]
    pub(crate) quicklook_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<Text>,
}

impl Item {
    pub fn new(title: impl Into<String>) -> Self {
        Item {
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(Arg::One(arg.into()));
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.arg = Some(Arg::Many(args.into_iter().map(Into::into).collect()));
        self
    }

    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    pub fn valid(mut self, valid: bool) -> Self {
        self.valid = Some(valid);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_for_filetype(mut self, filetype: impl Into<String>) -> Self {
        self.icon = Some(Icon {
            type_: Some("filetype".to_string()),
            path: filetype.into(),
        });
        self
    }

    pub fn icon_from_image(mut self, path_to_image: impl Into<String>) -> Self {
        self.icon = Some(Icon {
            type_: None,
            path: path_to_image.into(),
        });
        self
    }

    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.insert(modifier.keys.clone(), modifier);
        self
    }

    pub fn autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.autocomplete = Some(autocomplete.into());
        self
    }

    pub fn matches(mut self, matches: impl Into<String>) -> Self {
        self.r#match = Some(matches.into());
        self
    }

    pub fn quicklook_url(mut self, url: impl Into<String>) -> Self {
        self.quicklook_url = Some(url.into());
        self
    }

    pub fn copy_text(mut self, text: impl Into<String>) -> Self {
        self.text.get_or_insert_with(Text::default).copy = Some(text.into());
        self
    }

    pub fn large_type_text(mut self, text: impl Into<String>) -> Self {
        self.text.get_or_insert_with(Text::default).large_type = Some(text.into());
        self
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;
    use crate::ICON_TOOLBAR_FAVORITES;

    #[test]
    fn test_arg() {
        let item = Item::new("Item").arg("singlearg");
        let json = serde_json::to_value(&item).unwrap();
        let expected = json!({
            "title": "Item",
            "arg": "singlearg"
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_args() {
        let item = Item::new("Item").args(["arg1", "arg2", "https://www.google.com"]);
        let json = serde_json::to_value(&item).unwrap();
        let expected = json!({
            "title": "Item",
            "arg": ["arg1", "arg2", "https://www.google.com"]
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_matches() {
        let item = Item::new("Item").matches("realitemname");
        let json = serde_json::to_value(&item).unwrap();
        let expected = json!({
                    "title": "Item",
                    "match": "realitemname"
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_copy_text() {
        let item = Item::new("Google").copy_text("www.google.com");
        assert_eq!(item.title, "Google");
        assert_eq!(item.text.unwrap().copy, Some("www.google.com".to_string()));
    }

    #[test]
    fn test_quicklook_url() {
        let item = Item::new("Google").quicklook_url("https://www.google.com");
        assert_eq!(item.title, "Google");
        assert_eq!(
            item.quicklook_url,
            Some("https://www.google.com".to_string())
        );
    }

    #[test]
    fn test_large_type_text() {
        let item = Item::new("Google").large_type_text("www.google.com");
        assert_eq!(item.title, "Google");
        assert_eq!(
            item.text.unwrap().large_type,
            Some("www.google.com".to_string())
        );
    }

    #[test]
    fn test_icon_from_string() {
        let modifier = Item::new("Favorite").icon(ICON_TOOLBAR_FAVORITES.into());
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "title": "Favorite",
            "icon": {
                "path": ICON_TOOLBAR_FAVORITES,
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_icon_from_image() {
        let item = Item::new("Adobe PDF").icon_from_image("/Users/crayons/Documents/acrobat.png");
        let icon = item.icon.unwrap();
        assert_eq!(icon.type_, None);
        assert_eq!(icon.path, "/Users/crayons/Documents/acrobat.png");
    }

    #[test]
    fn test_icon_for_filetype() {
        let item = Item::new("Adobe PDF").icon_for_filetype("com.adobe.pdf");
        let icon = item.icon.unwrap();
        assert_eq!(icon.type_.unwrap(), "filetype");
        assert_eq!(icon.path, "com.adobe.pdf");
    }
}
