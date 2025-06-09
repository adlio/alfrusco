use std::env::var;

use arboard::Clipboard;
use log::{debug, error, info};

use crate::{Error, Response, Result};

/// Handles clipboard operations based on environment variables.
/// Returns true if a clipboard operation was performed and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle_clipboard() -> bool {
    let cmd = var("ALFRUSCO_COMMAND").ok();
    let title = var("TITLE").ok();
    let url = var("URL").ok();

    if let Some(cmd) = cmd {
        debug!("ALFRUSCO_COMMAND provided: {cmd}");

        if cmd == "richtext" || cmd == "markdown" {
            if let (Some(title), Some(url)) = (title, url) {
                let result = if cmd == "richtext" {
                    copy_rich_text_link_to_clipboard(title, url)
                } else {
                    copy_markdown_link_to_clipboard(title, url)
                };

                if let Err(e) = result {
                    error!("Clipboard operation failed: {e}");
                }

                // Write response and indicate that the process should exit
                if let Err(e) = Response::new().write(std::io::stdout()) {
                    error!("Error writing response: {e}");
                }
                return true;
            }
        }
    }

    // No clipboard operation was performed
    false
}

/// Copy a Markdown link to the clipboard.
/// Format: [title](url)
pub fn copy_markdown_link_to_clipboard(
    title: impl Into<String>,
    url: impl Into<String>,
) -> Result<()> {
    let title = title.into();
    let url = url.into();
    let markdown = format!("[{title}]({url})");

    let mut ctx = Clipboard::new()
        .map_err(|e| Error::Config(format!("Failed to initialize clipboard: {e}")))?;
    ctx.set_text(&markdown)
        .map_err(|e| Error::Config(format!("Failed to set clipboard text: {e}")))?;

    info!("Wrote Markdown link to clipboard: {markdown}");
    Ok(())
}

/// Copy a rich text link to the clipboard.
/// Format: <a href="url">title</a>
pub fn copy_rich_text_link_to_clipboard(
    title: impl Into<String>,
    url: impl Into<String>,
) -> Result<()> {
    let title = title.into();
    let url = url.into();

    // Create HTML and plain text versions
    let html = format!("<a href=\"{url}\">{title}</a>");
    let plain_text = format!("{title} ({url})");

    let mut ctx = Clipboard::new()
        .map_err(|e| Error::Config(format!("Failed to initialize clipboard: {e}")))?;
    ctx.set_html(&html, Some(&plain_text))
        .map_err(|e| Error::Config(format!("Failed to set clipboard HTML: {e}")))?;

    info!("Wrote HTML link to clipboard with fallback text");
    debug!("HTML: {html}");
    debug!("Plain text: {plain_text}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::Once;

    use temp_env::with_vars;

    use super::*;

    // Initialize test environment
    static INIT: Once = Once::new();
    fn initialize() {
        INIT.call_once(|| {
            env::set_var("RUST_LOG", "debug");
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }

    #[test]
    fn test_handle_clipboard_markdown() {
        initialize();
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard();
                assert!(
                    result,
                    "handle_clipboard should return true for markdown command"
                );
            },
        );
    }

    #[test]
    fn test_handle_clipboard_richtext() {
        initialize();
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("richtext")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard();
                assert!(
                    result,
                    "handle_clipboard should return true for richtext command"
                );
            },
        );
    }

    #[test]
    fn test_handle_clipboard_missing_params() {
        initialize();
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", None),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard();
                assert!(
                    !result,
                    "handle_clipboard should return false when title is missing"
                );
            },
        );

        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", Some("Test Title")),
                ("URL", None),
            ],
            || {
                let result = handle_clipboard();
                assert!(
                    !result,
                    "handle_clipboard should return false when URL is missing"
                );
            },
        );
    }

    #[test]
    fn test_handle_clipboard_unknown_command() {
        initialize();
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("unsupported")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard();
                assert!(
                    !result,
                    "handle_clipboard should return false for unknown commands"
                );
            },
        );
    }

    #[test]
    fn test_markdown_link_formatting() {
        initialize();

        // Test basic formatting
        let result = copy_markdown_link_to_clipboard("Test", "https://example.com");
        assert!(result.is_ok(), "Basic markdown link should succeed");

        // Test with special characters
        let result = copy_markdown_link_to_clipboard(
            "Title [with] brackets",
            "https://example.com/path?q=test&p=1",
        );
        assert!(
            result.is_ok(),
            "Markdown link with special characters should succeed"
        );
    }

    #[test]
    fn test_rich_text_link_formatting() {
        initialize();

        // Test basic formatting
        let result = copy_rich_text_link_to_clipboard("Test", "https://example.com");
        assert!(result.is_ok(), "Basic rich text link should succeed");

        // Test with special characters that need HTML escaping
        let result = copy_rich_text_link_to_clipboard(
            "Title with \"quotes\" & ampersands",
            "https://example.com/path?q=test&p=1",
        );
        assert!(
            result.is_ok(),
            "Rich text link with special characters should succeed"
        );
    }
}