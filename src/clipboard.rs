use std::env::var;

use arboard::Clipboard;
use log::{debug, error, info};

use crate::error::{Error, Result};
use crate::response::Response;

/// Handle clipboard operations based on environment variables.
/// Returns true if a clipboard operation was performed, false otherwise.
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

/// Format a Markdown link.
/// Format: [title](url)
pub fn format_markdown_link(title: impl Into<String>, url: impl Into<String>) -> String {
    let title = title.into();
    let url = url.into();
    format!("[{title}]({url})")
}

/// Format a rich text HTML link.
/// Format: <a href="url">title</a>
pub fn format_html_link(title: impl Into<String>, url: impl Into<String>) -> String {
    let title = title.into();
    let url = url.into();
    format!("<a href=\"{url}\">{title}</a>")
}

/// Copy a Markdown link to the clipboard.
/// Format: [title](url)
pub fn copy_markdown_link_to_clipboard(
    title: impl Into<String>,
    url: impl Into<String>,
) -> Result<()> {
    let markdown = format_markdown_link(title, url);

    let mut ctx = Clipboard::new()
        .map_err(|e| Error::Clipboard(format!("Failed to initialize clipboard: {e}")))?;
    ctx.set_text(&markdown)
        .map_err(|e| Error::Clipboard(format!("Failed to set clipboard text: {e}")))?;

    info!("Wrote Markdown link to clipboard: {markdown}");
    Ok(())
}

/// Copy a rich text link to the clipboard.
/// Format: <a href="url">title</a>
pub fn copy_rich_text_link_to_clipboard(
    title: impl Into<String>,
    url: impl Into<String>,
) -> Result<()> {
    let html = format_html_link(title, url);

    let mut ctx = Clipboard::new()
        .map_err(|e| Error::Clipboard(format!("Failed to initialize clipboard: {e}")))?;
    ctx.set_html(&html, None)
        .map_err(|e| Error::Clipboard(format!("Failed to set clipboard HTML: {e}")))?;

    info!("Wrote rich text link to clipboard: {html}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::config::TestingProvider;
    use crate::logging::init_logging;

    fn initialize() {
        let temp_dir = tempdir().unwrap();
        let provider = TestingProvider(temp_dir.path().to_path_buf());
        let _ = init_logging(&provider);
    }

    // Helper function to clean up environment variables
    fn cleanup_env_vars() {
        std::env::remove_var("ALFRUSCO_COMMAND");
        std::env::remove_var("TITLE");
        std::env::remove_var("URL");
    }

    #[test]
    fn test_handle_clipboard_markdown() {
        initialize();
        cleanup_env_vars();

        std::env::set_var("ALFRUSCO_COMMAND", "markdown");
        std::env::set_var("TITLE", "Test Title");
        std::env::set_var("URL", "https://example.com");

        let result = handle_clipboard();
        assert!(
            result,
            "handle_clipboard should return true for markdown command"
        );

        cleanup_env_vars();
    }

    #[test]
    fn test_handle_clipboard_richtext() {
        initialize();
        cleanup_env_vars();

        std::env::set_var("ALFRUSCO_COMMAND", "richtext");
        std::env::set_var("TITLE", "Test Title");
        std::env::set_var("URL", "https://example.com");

        let result = handle_clipboard();
        assert!(
            result,
            "handle_clipboard should return true for richtext command"
        );

        cleanup_env_vars();
    }

    #[test]
    fn test_handle_clipboard_missing_params() {
        initialize();
        cleanup_env_vars();

        std::env::set_var("ALFRUSCO_COMMAND", "markdown");
        // Don't set TITLE or URL - they should be missing

        let result = handle_clipboard();
        assert!(
            !result,
            "handle_clipboard should return false for missing params"
        );

        cleanup_env_vars();
    }

    #[test]
    fn test_handle_clipboard_unknown_command() {
        initialize();
        cleanup_env_vars();

        std::env::set_var("ALFRUSCO_COMMAND", "unknown");
        std::env::set_var("TITLE", "Test Title");
        std::env::set_var("URL", "https://example.com");

        let result = handle_clipboard();
        assert!(
            !result,
            "handle_clipboard should return false for unknown commands"
        );

        cleanup_env_vars();
    }

    // Pure function tests - fast, deterministic, no side effects
    #[test]
    fn test_format_markdown_link() {
        assert_eq!(
            format_markdown_link("Test Title", "https://example.com"),
            "[Test Title](https://example.com)"
        );

        assert_eq!(
            format_markdown_link(
                "Title [with] brackets",
                "https://example.com/path?q=test&p=1"
            ),
            "[Title [with] brackets](https://example.com/path?q=test&p=1)"
        );

        assert_eq!(format_markdown_link("", ""), "[]()");
    }

    #[test]
    fn test_format_html_link() {
        assert_eq!(
            format_html_link("Test Title", "https://example.com"),
            "<a href=\"https://example.com\">Test Title</a>"
        );

        assert_eq!(
            format_html_link("Title <with> HTML", "https://example.com/path?q=test&p=1"),
            "<a href=\"https://example.com/path?q=test&p=1\">Title <with> HTML</a>"
        );

        assert_eq!(format_html_link("", ""), "<a href=\"\"></a>");
    }

    // Integration tests - these access the actual clipboard
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

        // Test with special characters
        let result = copy_rich_text_link_to_clipboard(
            "Title <with> HTML",
            "https://example.com/path?q=test&p=1",
        );
        assert!(
            result.is_ok(),
            "Rich text link with special characters should succeed"
        );
    }
}
