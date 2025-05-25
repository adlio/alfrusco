use std::env;

use alfrusco::clipboard::{copy_markdown_link_to_clipboard, copy_rich_text_link_to_clipboard, handle_clipboard_internal};

#[test]
fn test_copy_markdown_link_to_clipboard() {
    // This test will actually modify the system clipboard
    // We can verify the function runs without errors, but can't easily verify the clipboard content
    copy_markdown_link_to_clipboard("Test Title", "https://example.com");
    
    // The function should complete without panicking
}

#[test]
#[cfg(target_os = "macos")]
fn test_copy_rich_text_link_to_clipboard() {
    // This test will only run on macOS since it uses osascript
    // We can verify the function runs without errors
    copy_rich_text_link_to_clipboard("Test Title", "https://example.com");
    
    // The function should complete without panicking
}

#[test]
fn test_handle_clipboard_internal_markdown() {
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, Some(0));
        },
    );
}

#[test]
fn test_handle_clipboard_internal_richtext() {
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("richtext")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, Some(0));
        },
    );
}

#[test]
fn test_handle_clipboard_internal_no_command() {
    // Set up environment variables for the test with no ALFRUSCO_COMMAND
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", None),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, None);
        },
    );
}

#[test]
fn test_handle_clipboard_internal_missing_title() {
    // Set up environment variables for the test with missing title
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", None),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, None);
        },
    );
}

#[test]
fn test_handle_clipboard_internal_missing_url() {
    // Set up environment variables for the test with missing URL
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", None),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, None);
        },
    );
}

#[test]
fn test_handle_clipboard_internal_unknown_command() {
    // Set up environment variables for the test with unknown command
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("unknown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard_internal();
            assert_eq!(result, None);
        },
    );
}
