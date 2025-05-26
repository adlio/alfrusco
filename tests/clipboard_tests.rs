use std::sync::Once;
use std::{env, io};

use alfrusco::clipboard::{
    copy_markdown_link_to_clipboard, copy_rich_text_link_to_clipboard, handle_clipboard,
};
use alfrusco::Response;

// Initialize test environment
static INIT: Once = Once::new();
fn initialize() {
    INIT.call_once(|| {
        env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

#[test]
fn test_copy_markdown_link_to_clipboard() {
    initialize();
    // This test will actually modify the system clipboard
    // We can verify the function runs without errors, but can't easily verify the clipboard content
    copy_markdown_link_to_clipboard("Test Title", "https://example.com");

    // The function should complete without panicking
}

#[test]
#[cfg(target_os = "macos")]
fn test_copy_rich_text_link_to_clipboard() {
    initialize();
    // This test will only run on macOS since it uses osascript
    // We can verify the function runs without errors
    copy_rich_text_link_to_clipboard("Test Title", "https://example.com");

    // The function should complete without panicking
}

#[test]
fn test_handle_clipboard_markdown() {
    initialize();
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Test the internal function that doesn't call exit()
            let result = handle_clipboard();
            assert!(result);
        },
    );
}

#[test]
fn test_handle_internal_error_path() {
    initialize();
    // Set up environment variables for the test
    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            // Create a test that simulates an error when writing the response
            struct FailingWriter;
            impl io::Write for FailingWriter {
                fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                    Err(io::Error::other("Simulated write error"))
                }
                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }
            }

            // Create a response and try to write it to our failing writer
            let response = Response::new();
            let result = response.write(FailingWriter);
            assert!(result.is_err());
        },
    );
}
