use std::collections::HashMap;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum Arg {
    One(String),
    Many(Vec<String>),
}

pub enum Key {
    Cmd,
    Ctrl,
    Alt,
    Shift,
    Fn,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Modifier {
    #[serde(skip_serializing)]
    keys: String,

    subtitle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<Arg>,

    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon>,

    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    autocomplete: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,
}

/// Item represents a single choice in the Alfred UI. The fields here
/// are designed around the Script Filter JSON format defined on
/// the Alfred web site
/// (https://www.alfredapp.com/help/workflows/inputs/script-filter/json/).
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

    pub fn r#match(mut self, matches: impl Into<String>) -> Self {
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
        self.subtitle = subtitle.into();
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub(crate) type_: Option<String>,

    pub(crate) path: String,
}

/// Text defines the two text options for an Alfred Item. copy is the text
/// that is copied to the clipboard when the user pressed CMD-C.largetype
/// is the text that is displayed in large type when the user pressed CMD-:.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) copy: Option<String>,

    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    pub(crate) large_type: Option<String>,
}

pub fn filter_and_sort_items(items: Vec<Item>, query: String) -> Result<Vec<Item>> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_items: Vec<(Item, i64)> = items
        .into_iter()
        .filter_map(|item| {
            matcher
                .fuzzy_match(&item.title, &query)
                .map(|score| (item, score))
        })
        .collect();

    // Sort by score in descending order
    filtered_items.sort_by(|a, b| b.1.cmp(&a.1));

    let items = filtered_items.into_iter().map(|(item, _)| item).collect();
    Ok(items)
}
