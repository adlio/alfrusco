use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Item {
    title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<Arg>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    variables: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon>,

    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    matches: Option<String>,

    #[serde(rename = "mods", skip_serializing_if = "HashMap::is_empty")]
    modifiers: HashMap<String, Modifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    autocomplete: Option<String>,

    #[serde(rename = "quicklookurl", skip_serializing_if = "Option::is_none")]
    quicklook_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Text>,
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
        self.matches = Some(matches.into());
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
    type_: Option<String>,

    path: String,
}

/// Text defines the two text options for an Alfred Item. copy is the text
/// that is copied to the clipboard when the user pressed CMD-C.largetype
/// is the text that is displayed in large type when the user pressed CMD-:.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<String>,

    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    large_type: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct URLItem {
    title: String,
    url: String,
    short_title: Option<String>,
    long_title: Option<String>,
    icon: Option<Icon>,
    display_title: Option<String>,
    copy_text: Option<String>,
}

impl URLItem {
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        URLItem {
            title: title.into(),
            url: url.into(),
            ..Self::default()
        }
    }

    pub fn short_title(mut self, short_title: impl Into<String>) -> Self {
        self.short_title = Some(short_title.into());
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

    pub fn display_title(mut self, display_title: impl Into<String>) -> Self {
        self.display_title = Some(display_title.into());
        self
    }

    pub fn long_title(mut self, long_title: impl Into<String>) -> Self {
        self.long_title = Some(long_title.into());
        self
    }

    pub fn copy_text(mut self, copy_text: impl Into<String>) -> Self {
        self.copy_text = Some(copy_text.into());
        self
    }
}

impl From<URLItem> for Item {
    fn from(url_item: URLItem) -> Self {
        let display_title = match url_item.display_title {
            Some(dt) => dt,
            None => url_item.title.clone(),
        };
        let title = url_item.title.clone();
        let short_title = url_item.short_title.clone();
        let long_title = url_item.long_title.clone();
        let url = url_item.url.clone();
        let copy_text = url_item.copy_text.clone();

        let cmd_mod = Modifier::new(Key::Cmd)
            .subtitle(format!("Copy Markdown Link '{}'", &title))
            .arg("run")
            .var("ALFRUSCO_COMMAND", "markdown")
            .var("TITLE", &title)
            .var("URL", &url);
        let alt_mod = Modifier::new(Key::Alt)
            .subtitle(format!("Copy Rich Text Link '{}'", &title))
            .arg("run")
            .var("ALFRUSCO_COMMAND", "richtext")
            .var("TITLE", &title)
            .var("URL", &url);

        let mut item = Item::new(display_title)
            .subtitle(&url_item.url)
            .uid(&url_item.url)
            .arg(&url_item.url)
            .copy_text(&url_item.url)
            .valid(true)
            .modifier(cmd_mod)
            .modifier(alt_mod);

        if url_item.icon.is_some() {
            item = item.icon(url_item.icon.unwrap());
        }

        match &short_title {
            Some(short_title) => {
                item = item
                    .modifier(
                        Modifier::new_combo(&[Key::Cmd, Key::Shift])
                            .subtitle(format!("Copy Markdown Link '{}'", short_title))
                            .arg("run")
                            .var("ALFRUSCO_COMMAND", "markdown")
                            .var("TITLE", short_title)
                            .var("URL", &url)
                            .valid(true),
                    )
                    .modifier(
                        Modifier::new_combo(&[Key::Alt, Key::Shift])
                            .subtitle(format!("Copy Rich Text Link '{}'", short_title))
                            .arg("run")
                            .var("ALFRUSCO_COMMAND", "richtext")
                            .var("TITLE", short_title)
                            .var("URL", &url)
                            .valid(true),
                    )
            }
            None => {}
        }

        match &long_title {
            Some(long_title) => {
                item = item
                    .modifier(
                        Modifier::new_combo(&[Key::Cmd, Key::Ctrl])
                            .subtitle(format!("Copy Markdown Link '{}'", long_title))
                            .arg("run")
                            .var("ALFRUSCO_COMMAND", "markdown")
                            .var("TITLE", long_title)
                            .var("URL", &url)
                            .valid(true),
                    )
                    .modifier(
                        Modifier::new_combo(&[Key::Alt, Key::Ctrl])
                            .subtitle(format!("Copy Rich Text Link '{}'", long_title))
                            .arg("run")
                            .var("ALFRUSCO_COMMAND", "richtext")
                            .var("TITLE", long_title)
                            .var("URL", &url)
                            .valid(true),
                    );
            }
            None => {}
        }

        if let Some(copy_text) = copy_text {
            item = item.copy_text(copy_text);
        }

        item
    }
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
