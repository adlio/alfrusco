use std::env::var;
use std::process::Command;

use clipboard::{ClipboardContext, ClipboardProvider};
use hex::encode;
use log::{debug, info};

use crate::Response;

/// Handle clipboard operations based on environment variables.
/// This is the main entry point for clipboard operations.
pub fn handle_clipboard() {
    let result = handle_clipboard_internal();
    
    // If the operation was successful and requires exiting, exit the process
    if let Some(exit_code) = result {
        std::process::exit(exit_code);
    }
}

/// Internal implementation of handle_clipboard that doesn't call exit().
/// Returns Some(exit_code) if the process should exit, None otherwise.
/// This separation makes the function testable.
pub fn handle_clipboard_internal() -> Option<i32> {
    let cmd = var("ALFRUSCO_COMMAND").ok();
    let title = var("TITLE").ok();
    let url = var("URL").ok();
    
    if let Some(cmd) = cmd {
        debug!("ALFRUSCO_COMMAND provided. Alfrusco will handle this request");

        if cmd == "richtext" || cmd == "markdown" {
            if let (Some(title), Some(url)) = (title, url) {
                if cmd == "richtext" {
                    copy_rich_text_link_to_clipboard(title, url);
                } else if cmd == "markdown" {
                    copy_markdown_link_to_clipboard(title, url);
                }
                
                // Write response and indicate that the process should exit
                if let Err(e) = Response::new().write(std::io::stdout()) {
                    eprintln!("Error writing response: {}", e);
                    return Some(1);
                }
                return Some(0);
            }
        }
    }
    
    // No clipboard operation was performed
    None
}

/// Copy a markdown link to the clipboard.
/// Format: [title](url)
pub fn copy_markdown_link_to_clipboard(title: impl Into<String>, url: impl Into<String>) {
    let markdown = format!("[{}]({})", title.into(), url.into());
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(markdown.clone()).unwrap();
    info!("wrote Markdown: {markdown} to the clipboard");
}

/// Copy a rich text link to the clipboard.
/// Format: <a href="url">title</a>
pub fn copy_rich_text_link_to_clipboard(title: impl Into<String>, url: impl Into<String>) {
    let html = format!("<a href=\"{}\">{}</a>", url.into(), title.into());

    let apple_script = format!(
        "set the clipboard to {{text:\" \", «class HTML»:«data HTML{}»}}",
        encode(html.as_bytes()),
    );

    // Prepare and execute the osascript command
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&apple_script)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("osascript command failed: {stderr}");
    }

    info!("wrote HTML to the clipboard as rich text: {html}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::with_vars;
    
    #[test]
    fn test_handle_clipboard_internal_no_command() {
        with_vars(
            [
                ("ALFRUSCO_COMMAND", None),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard_internal();
                assert_eq!(result, None);
            },
        );
    }
    
    #[test]
    fn test_handle_clipboard_internal_markdown() {
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                // We can't fully test the clipboard operation in an automated test,
                // but we can verify that the function returns the expected exit code
                let result = handle_clipboard_internal();
                assert_eq!(result, Some(0));
            },
        );
    }
    
    #[test]
    fn test_handle_clipboard_internal_richtext() {
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("richtext")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                // We can't fully test the clipboard operation in an automated test,
                // but we can verify that the function returns the expected exit code
                let result = handle_clipboard_internal();
                assert_eq!(result, Some(0));
            },
        );
    }
    
    #[test]
    fn test_handle_clipboard_internal_missing_params() {
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", None),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard_internal();
                assert_eq!(result, None);
            },
        );
        
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", Some("Test Title")),
                ("URL", None),
            ],
            || {
                let result = handle_clipboard_internal();
                assert_eq!(result, None);
            },
        );
    }
    
    #[test]
    fn test_handle_clipboard_internal_unknown_command() {
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("unknown")),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let result = handle_clipboard_internal();
                assert_eq!(result, None);
            },
        );
    }
}
