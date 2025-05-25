use serde::{Deserialize, Serialize};

use crate::{Icon, Item, Key, Modifier};

#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct URLItem {
    title: String,
    subtitle: Option<String>,
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

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
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

        if url_item.subtitle.is_some() {
            item = item.subtitle(url_item.subtitle.unwrap());
        }

        if url_item.icon.is_some() {
            item = item.icon(url_item.icon.unwrap());
        }

        if let Some(short_title) = &short_title {
            item = item
                .modifier(
                    Modifier::new_combo(&[Key::Cmd, Key::Shift])
                        .subtitle(format!("Copy Markdown Link '{short_title}'"))
                        .arg("run")
                        .var("ALFRUSCO_COMMAND", "markdown")
                        .var("TITLE", short_title)
                        .var("URL", &url)
                        .valid(true),
                )
                .modifier(
                    Modifier::new_combo(&[Key::Alt, Key::Shift])
                        .subtitle(format!("Copy Rich Text Link '{short_title}'"))
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
                        .subtitle(format!("Copy Markdown Link '{long_title}'"))
                        .arg("run")
                        .var("ALFRUSCO_COMMAND", "markdown")
                        .var("TITLE", long_title)
                        .var("URL", &url)
                        .valid(true),
                )
                .modifier(
                    Modifier::new_combo(&[Key::Alt, Key::Ctrl])
                        .subtitle(format!("Copy Rich Text Link '{long_title}'"))
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
    use crate::Arg;

    #[test]
    fn test_new_url_item() {
        let item: Item = URLItem::new("Rust", "https://www.rust-lang.org/").into();
        assert_eq!(item.title, "Rust");
        assert_eq!(
            item.arg,
            Some(Arg::One("https://www.rust-lang.org/".to_string()))
        );
    }

    #[test]
    fn test_display_title_override() {
        let item: Item = URLItem::new("Rust", "https://www.rust-lang.org/")
            .display_title("Rust (Displayed in Alfred UI, but not used in links)")
            .into();
        assert_eq!(
            item.title,
            "Rust (Displayed in Alfred UI, but not used in links)"
        );
    }

    #[test]
    fn test_short_title_override() {
        let item: Item = URLItem::new("crates.io: Rust Package Repository", "https://crates.io/")
            .short_title("crates.io")
            .into();
        assert_eq!(item.title, "crates.io: Rust Package Repository");
        let lm = item.modifiers["cmd+shift"].clone();
        assert_eq!(
            lm.subtitle,
            Some("Copy Markdown Link 'crates.io'".to_string())
        );
    }

    #[test]
    fn test_long_title() {
        let item: Item = URLItem::new("Rust Blog", "https://blog.rust-lang.org/")
            .long_title("The Rust Programming Language Blog")
            .into();
        assert_eq!(item.title, "Rust Blog");
        let lm = item.modifiers["cmd+ctrl"].clone();
        assert_eq!(
            lm.subtitle,
            Some("Copy Markdown Link 'The Rust Programming Language Blog'".to_string()),
        );
        assert_eq!(lm.arg, Some(Arg::One("run".to_string())));
    }

    #[test]
    fn test_copy_text() {
        let item: Item = URLItem::new("Google", "https://www.google.com")
            .copy_text("www.google.com")
            .into();
        assert_eq!(item.title, "Google");
        assert_eq!(item.text.unwrap().copy, Some("www.google.com".to_string()));
    }

    #[test]
    fn test_icon_from_image() {
        let item: Item = URLItem::new("Adobe PDF", "https://www.adobe.com/acrobat.html")
            .icon_from_image("/Users/crayons/Documents/acrobat.png")
            .into();
        let icon = item.icon.unwrap();
        assert_eq!(icon.type_, None);
        assert_eq!(icon.path, "/Users/crayons/Documents/acrobat.png");
    }

    #[test]
    fn test_icon_for_filetype() {
        let item: Item = URLItem::new("Adobe PDF", "https://www.adobe.com/acrobat.html")
            .icon_for_filetype("com.adobe.pdf")
            .into();
        let icon = item.icon.unwrap();
        assert_eq!(icon.type_.unwrap(), "filetype");
        assert_eq!(icon.path, "com.adobe.pdf");
    }

    #[test]
    fn test_into_item() {
        let item: Item = URLItem::new("Rust", "https://www.rust-lang.org/").into();
        assert_eq!(item.title, "Rust");
    }
}
