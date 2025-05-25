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
