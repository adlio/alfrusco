use serde::{Deserialize, Serialize};

use crate::{Icon, Item, Key, Modifier};

#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct URLItem {
    title: String,
    subtitle: Option<String>,
    url: String,
    short_title: Option<String>,
    long_title: Option<String>,
    icon: Option<Icon>,
    display_title: Option<String>,
    copy_text: Option<String>,
    arg: Option<String>,
    variables: std::collections::HashMap<String, String>,
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

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self
    }

    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
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

        let arg_value = url_item.arg.as_ref().unwrap_or(&url_item.url);

        let mut item = Item::new(display_title)
            .subtitle(&url_item.url)
            .uid(&url_item.url)
            .arg(arg_value)
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

        // Add custom variables
        for (key, value) in url_item.variables {
            item = item.var(key, value);
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

    #[test]
    fn test_subtitle() {
        let url_item = URLItem::new("Rust", "https://www.rust-lang.org/")
            .subtitle("The Rust Programming Language");

        // Verify the subtitle is set correctly in the URLItem
        assert_eq!(
            url_item.subtitle,
            Some("The Rust Programming Language".to_string())
        );

        // Verify the subtitle is preserved when converted to Item
        let item: Item = url_item.into();
        assert_eq!(
            item.subtitle,
            Some("The Rust Programming Language".to_string())
        );
    }

    #[test]
    fn test_subtitle_override() {
        // Test that the subtitle method overrides the default URL subtitle
        let item: Item = URLItem::new("Rust", "https://www.rust-lang.org/")
            .subtitle("The Rust Programming Language")
            .into();

        // The subtitle should be the one we set, not the URL
        assert_eq!(
            item.subtitle,
            Some("The Rust Programming Language".to_string())
        );
        assert_ne!(
            item.subtitle,
            Some("https://www.rust-lang.org/".to_string())
        );
    }

    #[test]
    fn test_custom_arg() {
        let item: Item = URLItem::new("Search Results", "https://example.com/search")
            .arg("workflow:search:advanced")
            .into();

        assert_eq!(item.title, "Search Results");
        assert_eq!(
            item.arg,
            Some(Arg::One("workflow:search:advanced".to_string()))
        );
        // URL should still be preserved in subtitle and copy text
        assert_eq!(
            item.subtitle,
            Some("https://example.com/search".to_string())
        );
    }

    #[test]
    fn test_default_arg_behavior() {
        let item: Item = URLItem::new("Example", "https://example.com").into();

        // Without custom arg, should use URL as arg
        assert_eq!(item.arg, Some(Arg::One("https://example.com".to_string())));
    }

    #[test]
    fn test_var_support() {
        let item: Item = URLItem::new("Test Item", "https://example.com")
            .var("CUSTOM_VAR", "custom_value")
            .var("ANOTHER_VAR", "another_value")
            .into();

        assert_eq!(item.title, "Test Item");
        assert_eq!(
            item.variables.get("CUSTOM_VAR"),
            Some(&"custom_value".to_string())
        );
        assert_eq!(
            item.variables.get("ANOTHER_VAR"),
            Some(&"another_value".to_string())
        );
    }

    #[test]
    fn test_var_chaining() {
        let url_item = URLItem::new("Chained", "https://example.com")
            .subtitle("Test subtitle")
            .var("VAR1", "value1")
            .arg("custom_arg")
            .var("VAR2", "value2");

        let item: Item = url_item.into();

        assert_eq!(item.title, "Chained");
        assert_eq!(item.subtitle, Some("Test subtitle".to_string()));
        assert_eq!(item.arg, Some(Arg::One("custom_arg".to_string())));
        assert_eq!(item.variables.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(item.variables.get("VAR2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_short_title_with_alt_shift_modifier() {
        let item: Item = URLItem::new("Full Title", "https://example.com")
            .short_title("Short")
            .into();

        // Verify alt+shift modifier exists and is configured correctly
        let alt_shift_mod = item.modifiers.get("alt+shift").unwrap();
        assert_eq!(
            alt_shift_mod.subtitle,
            Some("Copy Rich Text Link 'Short'".to_string())
        );
        assert_eq!(alt_shift_mod.arg, Some(Arg::One("run".to_string())));
        assert_eq!(
            alt_shift_mod
                .variables
                .as_ref()
                .unwrap()
                .get("ALFRUSCO_COMMAND"),
            Some(&"richtext".to_string())
        );
        assert_eq!(
            alt_shift_mod.variables.as_ref().unwrap().get("TITLE"),
            Some(&"Short".to_string())
        );
        assert_eq!(
            alt_shift_mod.variables.as_ref().unwrap().get("URL"),
            Some(&"https://example.com".to_string())
        );
        assert_eq!(alt_shift_mod.valid, Some(true));
    }

    #[test]
    fn test_long_title_with_alt_ctrl_modifier() {
        let item: Item = URLItem::new("Short Title", "https://example.com")
            .long_title("Very Long Descriptive Title")
            .into();

        // Verify alt+ctrl modifier exists and is configured correctly
        let alt_ctrl_mod = item.modifiers.get("alt+ctrl").unwrap();
        assert_eq!(
            alt_ctrl_mod.subtitle,
            Some("Copy Rich Text Link 'Very Long Descriptive Title'".to_string())
        );
        assert_eq!(alt_ctrl_mod.arg, Some(Arg::One("run".to_string())));
        assert_eq!(
            alt_ctrl_mod
                .variables
                .as_ref()
                .unwrap()
                .get("ALFRUSCO_COMMAND"),
            Some(&"richtext".to_string())
        );
        assert_eq!(
            alt_ctrl_mod.variables.as_ref().unwrap().get("TITLE"),
            Some(&"Very Long Descriptive Title".to_string())
        );
        assert_eq!(
            alt_ctrl_mod.variables.as_ref().unwrap().get("URL"),
            Some(&"https://example.com".to_string())
        );
        assert_eq!(alt_ctrl_mod.valid, Some(true));
    }

    #[test]
    fn test_all_builder_methods_combined() {
        let item: Item = URLItem::new("Test", "https://example.com")
            .subtitle("Custom subtitle")
            .short_title("Short")
            .long_title("Long Title")
            .icon_for_filetype("public.text")
            .display_title("Display")
            .copy_text("Custom copy text")
            .arg("custom_arg")
            .var("VAR1", "value1")
            .var("VAR2", "value2")
            .var("VAR3", "value3")
            .into();

        // Verify all fields are set correctly
        assert_eq!(item.title, "Display");
        assert_eq!(item.subtitle, Some("Custom subtitle".to_string()));
        assert_eq!(item.arg, Some(Arg::One("custom_arg".to_string())));
        assert_eq!(
            item.text.as_ref().unwrap().copy,
            Some("Custom copy text".to_string())
        );

        // Verify icon
        let icon = item.icon.as_ref().unwrap();
        assert_eq!(icon.type_, Some("filetype".to_string()));
        assert_eq!(icon.path, "public.text");

        // Verify variables are all present
        assert_eq!(item.variables.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(item.variables.get("VAR2"), Some(&"value2".to_string()));
        assert_eq!(item.variables.get("VAR3"), Some(&"value3".to_string()));

        // Verify all modifiers exist with correct titles
        assert!(item.modifiers.contains_key("cmd"));
        assert!(item.modifiers.contains_key("alt"));
        assert!(item.modifiers.contains_key("cmd+shift"));
        assert!(item.modifiers.contains_key("alt+shift"));
        assert!(item.modifiers.contains_key("cmd+ctrl"));
        assert!(item.modifiers.contains_key("alt+ctrl"));
    }

    #[test]
    fn test_default_url_item() {
        let url_item = URLItem::default();
        assert_eq!(url_item.title, "");
        assert_eq!(url_item.subtitle, None);
        assert_eq!(url_item.url, "");
        assert_eq!(url_item.short_title, None);
        assert_eq!(url_item.long_title, None);
        assert_eq!(url_item.icon, None);
        assert_eq!(url_item.display_title, None);
        assert_eq!(url_item.copy_text, None);
        assert_eq!(url_item.arg, None);
        assert!(url_item.variables.is_empty());
    }

    #[test]
    fn test_url_item_clone_and_partial_eq() {
        let url_item1 = URLItem::new("Test", "https://example.com")
            .subtitle("Sub")
            .short_title("Short");

        let url_item2 = url_item1.clone();
        assert_eq!(url_item1, url_item2);

        let url_item3 = URLItem::new("Test", "https://example.com")
            .subtitle("Different Sub")
            .short_title("Short");
        assert_ne!(url_item1, url_item3);
    }

    #[test]
    fn test_url_item_debug_format() {
        let url_item = URLItem::new("Test", "https://example.com");
        let debug_str = format!("{url_item:?}");
        assert!(debug_str.contains("URLItem"));
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("https://example.com"));
    }

    #[test]
    fn test_into_conversion_without_subtitle() {
        let item: Item = URLItem::new("No Subtitle", "https://example.com").into();
        // Without subtitle set, should use URL as subtitle
        assert_eq!(item.subtitle, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_into_conversion_without_icon() {
        let item: Item = URLItem::new("No Icon", "https://example.com").into();
        assert_eq!(item.icon, None);
    }

    #[test]
    fn test_into_conversion_without_copy_text() {
        let item: Item = URLItem::new("Default Copy", "https://example.com").into();
        // Should use URL as copy text by default
        assert_eq!(
            item.text.as_ref().unwrap().copy,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_modifiers_preserve_original_title_in_variables() {
        let item: Item = URLItem::new("Original Title", "https://example.com")
            .display_title("Display Title")
            .into();

        // The cmd modifier should use the original title, not the display title
        let cmd_mod = item.modifiers.get("cmd").unwrap();
        assert_eq!(
            cmd_mod.variables.as_ref().unwrap().get("TITLE"),
            Some(&"Original Title".to_string())
        );
    }

    #[test]
    fn test_multiple_variables_iteration() {
        let item: Item = URLItem::new("Test", "https://example.com")
            .var("KEY1", "value1")
            .var("KEY2", "value2")
            .var("KEY3", "value3")
            .var("KEY4", "value4")
            .var("KEY5", "value5")
            .into();

        // Verify all custom variables are transferred
        assert_eq!(item.variables.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(item.variables.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(item.variables.get("KEY3"), Some(&"value3".to_string()));
        assert_eq!(item.variables.get("KEY4"), Some(&"value4".to_string()));
        assert_eq!(item.variables.get("KEY5"), Some(&"value5".to_string()));
    }

    #[test]
    fn test_short_and_long_title_together() {
        let item: Item = URLItem::new("Regular", "https://example.com")
            .short_title("S")
            .long_title("Long Long Title")
            .into();

        // Should have modifiers for both short and long titles
        assert!(item.modifiers.contains_key("cmd+shift"));
        assert!(item.modifiers.contains_key("alt+shift"));
        assert!(item.modifiers.contains_key("cmd+ctrl"));
        assert!(item.modifiers.contains_key("alt+ctrl"));

        // Verify short title modifiers
        let cmd_shift = item.modifiers.get("cmd+shift").unwrap();
        assert_eq!(
            cmd_shift.subtitle,
            Some("Copy Markdown Link 'S'".to_string())
        );

        // Verify long title modifiers
        let cmd_ctrl = item.modifiers.get("cmd+ctrl").unwrap();
        assert_eq!(
            cmd_ctrl.subtitle,
            Some("Copy Markdown Link 'Long Long Title'".to_string())
        );
    }

    #[test]
    fn test_builder_method_string_conversions() {
        // Test that builder methods accept both &str and String
        let string_title = String::from("String Title");
        let string_url = String::from("https://example.com");
        let string_subtitle = String::from("String Subtitle");

        let item: Item = URLItem::new(string_title, string_url)
            .subtitle(string_subtitle)
            .short_title(String::from("String Short"))
            .long_title(String::from("String Long"))
            .display_title(String::from("String Display"))
            .copy_text(String::from("String Copy"))
            .arg(String::from("string_arg"))
            .var(String::from("KEY"), String::from("value"))
            .into();

        assert_eq!(item.title, "String Display");
        assert_eq!(item.subtitle, Some("String Subtitle".to_string()));
    }

    #[test]
    fn test_empty_string_values() {
        let item: Item = URLItem::new("", "")
            .subtitle("")
            .short_title("")
            .long_title("")
            .into();

        assert_eq!(item.title, "");
        assert_eq!(item.subtitle, Some(String::new()));

        // Empty short_title and long_title should still create modifiers
        assert!(item.modifiers.contains_key("cmd+shift"));
        assert!(item.modifiers.contains_key("cmd+ctrl"));
    }

    #[test]
    fn test_icon_methods_mutually_exclusive() {
        // Test that icon_for_filetype overwrites icon_from_image
        let item: Item = URLItem::new("Test", "https://example.com")
            .icon_from_image("/path/to/image.png")
            .icon_for_filetype("com.adobe.pdf")
            .into();

        let icon = item.icon.as_ref().unwrap();
        assert_eq!(icon.type_, Some("filetype".to_string()));
        assert_eq!(icon.path, "com.adobe.pdf");

        // Test the reverse
        let item2: Item = URLItem::new("Test", "https://example.com")
            .icon_for_filetype("com.adobe.pdf")
            .icon_from_image("/path/to/image.png")
            .into();

        let icon2 = item2.icon.as_ref().unwrap();
        assert_eq!(icon2.type_, None);
        assert_eq!(icon2.path, "/path/to/image.png");
    }
}
