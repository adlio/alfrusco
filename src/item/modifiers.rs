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
/// https://www.alfredapp.com/help/workflows/inputs/script-filter/json/
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
            keys: format!("{}", key),
            ..Self::default()
        }
    }

    pub fn new_combo(keys: &[Key]) -> Self {
        Self {
            keys: keys
                .iter()
                .map(|key| format!("{}", key))
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
}
