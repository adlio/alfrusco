use crate::{Icon, Item, Key, Modifier};
use serde::{Deserialize, Serialize};

///
#[non_exhaustive]
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

        if let Some(short_title) = &short_title {
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

        if let Some(long_title) = &long_title {
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

        if let Some(copy_text) = copy_text {
            item = item.copy_text(copy_text);
        }

        item
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_url_item() {
        let url_item = URLItem::new("Rust", "https://www.rust-lang.org/");
        assert_eq!(url_item.title, "Rust");
        assert_eq!(url_item.url, "https://www.rust-lang.org/");
    }

    #[test]
    fn test_short_title_override() {
        let url_item = URLItem::new("crates.io: Rust Package Repository", "https://crates.io/")
            .short_title("crates.io");
        assert_eq!(url_item.title, "crates.io: Rust Package Repository");
        assert_eq!(url_item.short_title.unwrap(), "crates.io");
    }

    #[test]
    fn test_into_item() {
        let url_item = URLItem::new("Rust", "https://www.rust-lang.org/");
        let item: Item = url_item.into();
        assert_eq!(item.title, "Rust");
    }
}
