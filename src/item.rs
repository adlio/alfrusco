use std::collections::HashMap;

use serde::Serialize;

pub use crate::{Arg, Icon, Modifier, Text};

/// Slight boost to fuzzy match score (+25 points).
/// Use when you want a subtle preference for an item.
pub const BOOST_SLIGHT: i64 = 25;

/// Low boost to fuzzy match score (+50 points).
/// Use for minor preference that can still be overridden by better matches.
pub const BOOST_LOW: i64 = 50;

/// Moderate boost to fuzzy match score (+75 points).
/// Use for noticeable preference in search results.
pub const BOOST_MODERATE: i64 = 75;

/// High boost to fuzzy match score (+100 points).
/// Use for strong preference, will often rank above similar matches.
pub const BOOST_HIGH: i64 = 100;

/// Higher boost to fuzzy match score (+150 points).
/// Use for very strong preference, will rank above most other matches.
pub const BOOST_HIGHER: i64 = 150;

/// Highest boost to fuzzy match score (+200 points).
/// Use to effectively guarantee top ranking among non-sticky items.
pub const BOOST_HIGHEST: i64 = 200;

/// Item represents a single choice in the Alfred selection UI.
///
/// The fields here are designed around the Script Filter JSON format defined
/// on the Alfred website:
///
/// (<https://www.alfredapp.com/help/workflows/inputs/script-filter/json/>).
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

    #[serde(skip_serializing)]
    pub(crate) sticky: bool,

    /// Boost value added to the fuzzy match score for sorting.
    /// Higher values rank the item higher in results.
    #[serde(skip_serializing)]
    pub(crate) boost: i64,
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

    pub fn unset_var(mut self, key: impl Into<String>) -> Self {
        self.variables.remove(&key.into());
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

    pub fn sticky(mut self, is_sticky: bool) -> Self {
        self.sticky = is_sticky;
        self
    }

    /// Set a boost value that's added to the fuzzy match score.
    ///
    /// Higher values rank this item higher in filtered results. Fuzzy match
    /// scores typically range from 30-150, so boost values in that range will
    /// have a meaningful impact on ordering.
    ///
    /// Use the provided constants for common boost levels:
    /// - [`BOOST_SLIGHT`] (25) - Subtle preference
    /// - [`BOOST_LOW`] (50) - Minor preference
    /// - [`BOOST_MODERATE`] (75) - Noticeable preference
    /// - [`BOOST_HIGH`] (100) - Strong preference
    /// - [`BOOST_HIGHER`] (150) - Very strong preference
    /// - [`BOOST_HIGHEST`] (200) - Effectively guarantees top ranking
    ///
    /// Note: Boost only affects non-sticky items. Sticky items always appear
    /// first regardless of boost value.
    ///
    /// # Example
    ///
    /// ```
    /// use alfrusco::{Item, BOOST_HIGH};
    ///
    /// let item = Item::new("Preferred Result")
    ///     .subtitle("This will rank higher")
    ///     .boost(BOOST_HIGH);
    /// ```
    pub fn boost(mut self, boost: i64) -> Self {
        self.boost = boost;
        self
    }

    #[cfg(test)]
    pub(crate) fn test_helper_get_sticky(&self) -> bool {
        self.sticky
    }

    #[cfg(test)]
    pub(crate) fn test_helper_get_boost(&self) -> i64 {
        self.boost
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

    #[test]
    fn test_var_and_unset_var() {
        // First, add a variable
        let item = Item::new("Test Item")
            .var("key1", "value1")
            .var("key2", "value2");

        // Verify both variables are set
        assert_eq!(item.variables.get("key1"), Some(&"value1".to_string()));
        assert_eq!(item.variables.get("key2"), Some(&"value2".to_string()));

        // Now unset one variable
        let item = item.unset_var("key1");

        // Verify key1 is removed but key2 remains
        assert_eq!(item.variables.get("key1"), None);
        assert_eq!(item.variables.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_unset_var_nonexistent_key() {
        // Create an item with one variable
        let item = Item::new("Test Item").var("key1", "value1");

        // Verify the variable is set
        assert_eq!(item.variables.get("key1"), Some(&"value1".to_string()));

        // Try to unset a variable that doesn't exist
        let item = item.unset_var("nonexistent_key");

        // Verify the original variable is still there
        assert_eq!(item.variables.get("key1"), Some(&"value1".to_string()));

        // Verify the HashMap size hasn't changed
        assert_eq!(item.variables.len(), 1);
    }

    #[test]
    fn test_var_and_unset_var_serialization() {
        // Create an item with variables
        let item = Item::new("Test Item")
            .var("key1", "value1")
            .var("key2", "value2");

        // Serialize to JSON
        let json = serde_json::to_value(&item).unwrap();

        // Verify variables are included in the JSON
        let expected = json!({
            "title": "Test Item",
            "variables": {
                "key1": "value1",
                "key2": "value2"
            }
        });
        assert_eq!(json, expected);

        // Unset a variable
        let item = item.unset_var("key1");

        // Serialize to JSON again
        let json = serde_json::to_value(&item).unwrap();

        // Verify the updated variables are in the JSON
        let expected = json!({
            "title": "Test Item",
            "variables": {
                "key2": "value2"
            }
        });
        assert_eq!(json, expected);

        // Unset the last variable
        let item = item.unset_var("key2");

        // Serialize to JSON again
        let json = serde_json::to_value(&item).unwrap();

        // Verify variables field is omitted when empty
        let expected = json!({
            "title": "Test Item"
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_sticky() {
        // Default should be false
        let item = Item::new("Test Item");
        assert!(!item.test_helper_get_sticky());

        // Set to true
        let item = item.sticky(true);
        assert!(item.test_helper_get_sticky());

        // Set back to false
        let item = item.sticky(false);
        assert!(!item.test_helper_get_sticky());

        // Verify sticky is not serialized (it's marked with skip_serializing)
        let json = serde_json::to_value(&item).unwrap();
        let expected = json!({
            "title": "Test Item"
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_boost_builder() {
        // Default should be 0
        let item = Item::new("Test Item");
        assert_eq!(item.test_helper_get_boost(), 0);

        // Test setting boost values
        let item = Item::new("Test").boost(100);
        assert_eq!(item.test_helper_get_boost(), 100);

        // Test negative boost
        let item = Item::new("Test").boost(-50);
        assert_eq!(item.test_helper_get_boost(), -50);
    }

    #[test]
    fn test_boost_not_serialized() {
        // Boost should not appear in JSON output
        let item = Item::new("Test Item").boost(100);
        let json_str = serde_json::to_string(&item).unwrap();
        assert!(!json_str.contains("boost"));
    }
}
