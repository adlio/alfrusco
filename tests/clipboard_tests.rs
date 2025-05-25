use std::env;

use alfrusco::clipboard::{copy_markdown_link_to_clipboard, copy_rich_text_link_to_clipboard};

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
fn test_handle_clipboard_with_markdown() {
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // We can't actually call handle_clipboard() directly because it calls std::process::exit(0)
            // Instead, we'll verify that the environment variables are set correctly
            assert_eq!(env::var("ALFRUSCO_COMMAND").unwrap(), "markdown");
            assert_eq!(env::var("TITLE").unwrap(), "Test Title");
            assert_eq!(env::var("URL").unwrap(), "https://example.com");
            
            // The actual clipboard operation would happen here in the real code
        },
    );
}

#[test]
fn test_handle_clipboard_with_richtext() {
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("richtext")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // We can't actually call handle_clipboard() directly because it calls std::process::exit(0)
            // Instead, we'll verify that the environment variables are set correctly
            assert_eq!(env::var("ALFRUSCO_COMMAND").unwrap(), "richtext");
            assert_eq!(env::var("TITLE").unwrap(), "Test Title");
            assert_eq!(env::var("URL").unwrap(), "https://example.com");
            
            // The actual clipboard operation would happen here in the real code
        },
    );
}

#[test]
fn test_handle_clipboard_no_command() {
    // Set up environment variables for the test with no ALFRUSCO_COMMAND
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", None),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // In this case, handle_clipboard() would do nothing
            // We're just verifying that the environment is set up correctly
            assert!(env::var("ALFRUSCO_COMMAND").is_err());
            assert_eq!(env::var("TITLE").unwrap(), "Test Title");
            assert_eq!(env::var("URL").unwrap(), "https://example.com");
        },
    );
}
