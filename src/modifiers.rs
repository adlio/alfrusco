// Standard library improts
use std::collections::HashMap;

// Third-party imports
use serde::Serialize;

// Local imports
use crate::{Arg, Icon};

/// Key represents one of the modifier Keys (Cmd, Ctrl, etc)
///
/// These are used as the key in the mods object within an
/// Alfred Item.
pub enum Key {
    Cmd,
    Ctrl,
    Alt,
    Shift,
    Fn,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Key::Cmd => write!(f, "cmd"),
            Key::Ctrl => write!(f, "ctrl"),
            Key::Alt => write!(f, "alt"),
            Key::Shift => write!(f, "shift"),
            Key::Fn => write!(f, "fn"),
        }
    }
}

/// Modifier provides a data structure to represent an item in the
/// `mods` object within an Alfred item.
///
/// Each mod is indexed by a Key (such as `cmd`) or a combination
/// of Keys (such as `cmd+shift`).
///
/// See more on the spec on the Alfred site:
/// <https://www.alfredapp.com/help/workflows/inputs/script-filter/json/>
///
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Modifier {
    #[serde(skip_serializing)]
    pub keys: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg: Option<Arg>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autocomplete: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
}

impl Modifier {
    pub fn new(key: Key) -> Self {
        Self {
            keys: format!("{key}"),
            ..Self::default()
        }
    }

    pub fn new_combo(keys: &[Key]) -> Self {
        Self {
            keys: keys
                .iter()
                .map(|key| format!("{key}"))
                .collect::<Vec<String>>()
                .join("+"),
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

    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables
            .get_or_insert(HashMap::new())
            .insert(key.into(), value.into());
        self
    }

    pub fn autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.autocomplete = Some(autocomplete.into());
        self
    }

    pub fn valid(mut self, valid: bool) -> Self {
        self.valid = Some(valid);
        self
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;
    use crate::ICON_TOOLBAR_FAVORITES;

    #[test]
    fn test_new() {
        let modifier = Modifier::new(Key::Fn);
        assert_eq!(modifier.keys, "fn");
    }

    #[test]
    fn test_new_combo() {
        let cases = [
            (vec![Key::Cmd, Key::Shift], "cmd+shift"),
            (vec![Key::Ctrl, Key::Fn], "ctrl+fn"),
            (vec![Key::Ctrl], "ctrl"),
            (vec![Key::Ctrl, Key::Shift, Key::Fn], "ctrl+shift+fn"),
        ];
        for (keys, expected) in cases {
            let modifier = Modifier::new_combo(&keys);
            assert_eq!(modifier.keys, expected);
        }
    }

    #[test]
    fn test_arg() {
        let modifier = Modifier::new(Key::Cmd).arg("singlearg");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "arg": "singlearg" });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_args() {
        let modifier = Modifier::new(Key::Alt).args(["arg1", "arg2", "https://www.google.com"]);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "arg": ["arg1", "arg2", "https://www.google.com"]
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_icon_from_string() {
        let modifier = Modifier::new(Key::Cmd).icon(ICON_TOOLBAR_FAVORITES.into());
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "icon": {
                "path": ICON_TOOLBAR_FAVORITES,
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_icon_from_image() {
        let modifier =
            Modifier::new(Key::Cmd).icon_from_image("/Users/crayons/Documents/acrobat.png");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "icon": {
                "path": "/Users/crayons/Documents/acrobat.png"
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_icon_for_filetype() {
        let modifier = Modifier::new(Key::Ctrl).icon_for_filetype("com.adobe.pdf");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "icon": {
                "type": "filetype",
                "path": "com.adobe.pdf"
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_autocomplete() {
        let modifier = Modifier::new(Key::Cmd).autocomplete("mycompletion");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "autocomplete": "mycompletion" });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_subtitle() {
        let modifier = Modifier::new(Key::Cmd).subtitle("Press Cmd for this action");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "subtitle": "Press Cmd for this action" });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_subtitle_with_string() {
        let modifier = Modifier::new(Key::Alt).subtitle(String::from("Alt action"));
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "subtitle": "Alt action" });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_var_single() {
        let modifier = Modifier::new(Key::Cmd).var("MY_VAR", "my_value");
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "variables": {
                "MY_VAR": "my_value"
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_var_multiple() {
        let modifier = Modifier::new(Key::Shift)
            .var("VAR1", "value1")
            .var("VAR2", "value2")
            .var("VAR3", "value3");
        let json = serde_json::to_value(&modifier).unwrap();

        // Verify all variables are present
        let vars = json.get("variables").unwrap().as_object().unwrap();
        assert_eq!(vars.get("VAR1").unwrap().as_str().unwrap(), "value1");
        assert_eq!(vars.get("VAR2").unwrap().as_str().unwrap(), "value2");
        assert_eq!(vars.get("VAR3").unwrap().as_str().unwrap(), "value3");
    }

    #[test]
    fn test_var_with_string_types() {
        let modifier =
            Modifier::new(Key::Fn).var(String::from("STRING_KEY"), String::from("string_value"));
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({
            "variables": {
                "STRING_KEY": "string_value"
            }
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_valid_true() {
        let modifier = Modifier::new(Key::Cmd).valid(true);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "valid": true });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_valid_false() {
        let modifier = Modifier::new(Key::Alt).valid(false);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "valid": false });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_key_display_all_variants() {
        let cases = [
            (Key::Cmd, "cmd"),
            (Key::Ctrl, "ctrl"),
            (Key::Alt, "alt"),
            (Key::Shift, "shift"),
            (Key::Fn, "fn"),
        ];

        for (key, expected) in cases {
            assert_eq!(format!("{key}"), expected);
        }
    }

    #[test]
    fn test_all_builder_methods_combined() {
        let modifier = Modifier::new(Key::Cmd)
            .subtitle("Complete action")
            .arg("action_arg")
            .icon_for_filetype("com.apple.folder")
            .var("VAR1", "value1")
            .var("VAR2", "value2")
            .autocomplete("complete_text")
            .valid(true);

        let json = serde_json::to_value(&modifier).unwrap();

        assert_eq!(
            json.get("subtitle").unwrap().as_str().unwrap(),
            "Complete action"
        );
        assert_eq!(json.get("arg").unwrap().as_str().unwrap(), "action_arg");
        assert_eq!(
            json.get("autocomplete").unwrap().as_str().unwrap(),
            "complete_text"
        );
        assert!(json.get("valid").unwrap().as_bool().unwrap());

        let icon = json.get("icon").unwrap();
        assert_eq!(icon.get("type").unwrap().as_str().unwrap(), "filetype");
        assert_eq!(
            icon.get("path").unwrap().as_str().unwrap(),
            "com.apple.folder"
        );

        let vars = json.get("variables").unwrap().as_object().unwrap();
        assert_eq!(vars.get("VAR1").unwrap().as_str().unwrap(), "value1");
        assert_eq!(vars.get("VAR2").unwrap().as_str().unwrap(), "value2");
    }

    #[test]
    fn test_modifier_default() {
        let modifier = Modifier::default();
        assert_eq!(modifier.keys, "");
        assert_eq!(modifier.subtitle, None);
        assert_eq!(modifier.arg, None);
        assert_eq!(modifier.icon, None);
        assert_eq!(modifier.variables, None);
        assert_eq!(modifier.autocomplete, None);
        assert_eq!(modifier.valid, None);
    }

    #[test]
    fn test_modifier_clone_and_eq() {
        let modifier1 = Modifier::new(Key::Cmd).subtitle("Test").var("KEY", "value");

        let modifier2 = modifier1.clone();
        assert_eq!(modifier1, modifier2);

        let modifier3 = Modifier::new(Key::Cmd).subtitle("Different");
        assert_ne!(modifier1, modifier3);
    }

    #[test]
    fn test_modifier_debug_format() {
        let modifier = Modifier::new(Key::Alt).subtitle("Debug test");
        let debug_str = format!("{modifier:?}");
        assert!(debug_str.contains("Modifier"));
        assert!(debug_str.contains("alt"));
    }

    #[test]
    fn test_keys_not_serialized() {
        let modifier = Modifier::new(Key::Cmd);
        let json = serde_json::to_value(&modifier).unwrap();
        // keys field should be skipped in serialization
        assert!(json.get("keys").is_none());
    }

    #[test]
    fn test_none_fields_not_serialized() {
        let modifier = Modifier::new(Key::Shift);
        let json = serde_json::to_value(&modifier).unwrap();

        // All None fields should be skipped
        assert!(json.get("subtitle").is_none());
        assert!(json.get("arg").is_none());
        assert!(json.get("icon").is_none());
        assert!(json.get("variables").is_none());
        assert!(json.get("autocomplete").is_none());
        assert!(json.get("valid").is_none());

        // Should be an empty object
        assert_eq!(json.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_combo_with_empty_slice() {
        let modifier = Modifier::new_combo(&[]);
        assert_eq!(modifier.keys, "");
    }

    #[test]
    fn test_combo_with_single_key() {
        let modifier = Modifier::new_combo(&[Key::Cmd]);
        assert_eq!(modifier.keys, "cmd");
    }

    #[test]
    fn test_combo_with_four_keys() {
        let modifier = Modifier::new_combo(&[Key::Cmd, Key::Ctrl, Key::Alt, Key::Shift]);
        assert_eq!(modifier.keys, "cmd+ctrl+alt+shift");
    }

    #[test]
    fn test_combo_with_all_keys() {
        let modifier = Modifier::new_combo(&[Key::Cmd, Key::Ctrl, Key::Alt, Key::Shift, Key::Fn]);
        assert_eq!(modifier.keys, "cmd+ctrl+alt+shift+fn");
    }

    #[test]
    fn test_icon_methods_on_modifier() {
        // Test icon_for_filetype
        let mod1 = Modifier::new(Key::Cmd).icon_for_filetype("public.text");
        assert_eq!(
            mod1.icon.as_ref().unwrap().type_,
            Some("filetype".to_string())
        );
        assert_eq!(mod1.icon.as_ref().unwrap().path, "public.text");

        // Test icon_from_image
        let mod2 = Modifier::new(Key::Alt).icon_from_image("/path/to/icon.png");
        assert_eq!(mod2.icon.as_ref().unwrap().type_, None);
        assert_eq!(mod2.icon.as_ref().unwrap().path, "/path/to/icon.png");

        // Test icon() with Icon struct
        let icon = Icon {
            type_: Some("fileicon".to_string()),
            path: "/custom/path".to_string(),
        };
        let mod3 = Modifier::new(Key::Ctrl).icon(icon);
        assert_eq!(
            mod3.icon.as_ref().unwrap().type_,
            Some("fileicon".to_string())
        );
        assert_eq!(mod3.icon.as_ref().unwrap().path, "/custom/path");
    }

    #[test]
    fn test_chaining_all_methods() {
        let modifier = Modifier::new_combo(&[Key::Cmd, Key::Shift])
            .subtitle("Chained subtitle")
            .arg("chained_arg")
            .icon_from_image("icon.png")
            .var("KEY1", "val1")
            .autocomplete("chain")
            .valid(false)
            .var("KEY2", "val2");

        assert_eq!(modifier.keys, "cmd+shift");
        assert_eq!(modifier.subtitle, Some("Chained subtitle".to_string()));
        assert_eq!(modifier.arg, Some(Arg::One("chained_arg".to_string())));
        assert_eq!(modifier.icon.as_ref().unwrap().path, "icon.png");
        assert_eq!(modifier.autocomplete, Some("chain".to_string()));
        assert_eq!(modifier.valid, Some(false));

        let vars = modifier.variables.as_ref().unwrap();
        assert_eq!(vars.get("KEY1"), Some(&"val1".to_string()));
        assert_eq!(vars.get("KEY2"), Some(&"val2".to_string()));
    }

    #[test]
    fn test_args_method_with_vec() {
        let modifier = Modifier::new(Key::Ctrl).args(vec!["arg1", "arg2", "arg3"]);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "arg": ["arg1", "arg2", "arg3"] });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_args_method_with_array() {
        let modifier = Modifier::new(Key::Fn).args(["first", "second"]);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "arg": ["first", "second"] });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_args_method_with_strings() {
        let modifier =
            Modifier::new(Key::Alt).args(vec![String::from("string1"), String::from("string2")]);
        let json = serde_json::to_value(&modifier).unwrap();
        let expected = json!({ "arg": ["string1", "string2"] });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_var_overwrites_existing_key() {
        let modifier = Modifier::new(Key::Cmd)
            .var("KEY", "first_value")
            .var("KEY", "second_value");

        let vars = modifier.variables.as_ref().unwrap();
        assert_eq!(vars.get("KEY"), Some(&"second_value".to_string()));
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn test_empty_string_values_in_methods() {
        let modifier = Modifier::new(Key::Shift)
            .subtitle("")
            .arg("")
            .var("", "")
            .autocomplete("");

        assert_eq!(modifier.subtitle, Some(String::new()));
        assert_eq!(modifier.arg, Some(Arg::One(String::new())));
        assert_eq!(modifier.autocomplete, Some(String::new()));

        let vars = modifier.variables.as_ref().unwrap();
        assert_eq!(vars.get(""), Some(&String::new()));
    }

    #[test]
    fn test_modifier_partial_eq_all_fields() {
        let mod1 = Modifier::new(Key::Cmd)
            .subtitle("Sub")
            .arg("arg")
            .icon_for_filetype("type")
            .var("K", "V")
            .autocomplete("auto")
            .valid(true);

        let mod2 = Modifier::new(Key::Cmd)
            .subtitle("Sub")
            .arg("arg")
            .icon_for_filetype("type")
            .var("K", "V")
            .autocomplete("auto")
            .valid(true);

        assert_eq!(mod1, mod2);
    }

    #[test]
    fn test_icon_from_const() {
        let modifier = Modifier::new(Key::Cmd).icon(ICON_TOOLBAR_FAVORITES.into());
        assert_eq!(modifier.icon.as_ref().unwrap().type_, None);
        assert_eq!(modifier.icon.as_ref().unwrap().path, ICON_TOOLBAR_FAVORITES);
    }
}
