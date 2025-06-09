use std::sync::Once;
use std::{env, io};

use alfrusco::clipboard::{
    copy_markdown_link_to_clipboard, copy_rich_text_link_to_clipboard, handle_clipboard,
};
use alfrusco::{Error, Response};

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

    let result = copy_markdown_link_to_clipboard("Test Title", "https://example.com");
    match result {
        Ok(_) => {
            // Success case - clipboard operation worked
        }
        Err(e) => {
            // In headless/CI environments, clipboard operations may fail
            println!(
                "Clipboard operation failed (expected in headless environment): {e}"
            );
            assert!(matches!(e, Error::Config(_)));
        }
    }

    let result = copy_markdown_link_to_clipboard(
        String::from("String Title"),
        String::from("https://example.com"),
    );
    match result {
        Ok(_) => {}
        Err(e) => {
            println!(
                "Clipboard operation failed (expected in headless environment): {e}"
            );
            assert!(matches!(e, Error::Config(_)));
        }
    }

    let result = copy_markdown_link_to_clipboard(
        "Title [with] special chars",
        "https://example.com/path?param=value&other=test",
    );
    match result {
        Ok(_) => {}
        Err(e) => {
            println!(
                "Clipboard operation failed (expected in headless environment): {e}"
            );
            assert!(matches!(e, Error::Config(_)));
        }
    }
}

#[test]
fn test_copy_rich_text_link_to_clipboard() {
    initialize();

    let result = copy_rich_text_link_to_clipboard("Test Title", "https://example.com");
    match result {
        Ok(_) => {
            // Success case - clipboard operation worked
        }
        Err(e) => {
            println!(
                "Clipboard operation failed (expected in headless environment): {e}"
            );
            assert!(matches!(e, Error::Config(_)));
        }
    }

    let result = copy_rich_text_link_to_clipboard(
        "Title with \"quotes\" & <tags>",
        "https://example.com/path?param=value&other=test",
    );
    match result {
        Ok(_) => {}
        Err(e) => {
            println!(
                "Clipboard operation failed (expected in headless environment): {e}"
            );
            assert!(matches!(e, Error::Config(_)));
        }
    }
}

#[test]
fn test_handle_clipboard_markdown() {
    initialize();

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(
                result,
                "Should return true when markdown clipboard operation is performed"
            );
        },
    );
}

#[test]
fn test_handle_clipboard_richtext() {
    initialize();

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("richtext")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(
                result,
                "Should return true when richtext clipboard operation is performed"
            );
        },
    );
}

#[test]
fn test_handle_clipboard_missing_environment_variables() {
    initialize();

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", None),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(
                !result,
                "Should return false when ALFRUSCO_COMMAND is missing"
            );
        },
    );

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", None),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(!result, "Should return false when TITLE is missing");
        },
    );

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", None),
        ],
        || {
            let result = handle_clipboard();
            assert!(!result, "Should return false when URL is missing");
        },
    );
}

#[test]
fn test_handle_clipboard_unsupported_command() {
    initialize();

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("unsupported")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(!result, "Should return false for unsupported commands");
        },
    );
}

#[test]
fn test_handle_clipboard_empty_values() {
    initialize();

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("")),
            ("URL", Some("https://example.com")),
        ],
        || {
            let result = handle_clipboard();
            assert!(result, "Should handle empty title gracefully");
        },
    );

    temp_env::with_vars(
        [
            ("ALFRUSCO_COMMAND", Some("markdown")),
            ("TITLE", Some("Test Title")),
            ("URL", Some("")),
        ],
        || {
            let result = handle_clipboard();
            assert!(result, "Should handle empty URL gracefully");
        },
    );
}

#[test]
fn test_response_write_error_handling() {
    initialize();

    struct FailingWriter;
    impl io::Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Simulated write error",
            ))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let response = Response::new();
    let result = response.write(FailingWriter);
    assert!(result.is_err(), "Write should fail with FailingWriter");
}

#[test]
fn test_unicode_and_special_characters() {
    initialize();

    let result = copy_markdown_link_to_clipboard("测试标题 🦀", "https://example.com/测试");
    match result {
        Ok(_) => {}
        Err(e) => {
            assert!(matches!(e, Error::Config(_)));
        }
    }

    let result = copy_rich_text_link_to_clipboard("测试标题 🦀", "https://example.com/测试");
    match result {
        Ok(_) => {}
        Err(e) => {
            assert!(matches!(e, Error::Config(_)));
        }
    }

    let special_chars = vec![
        ("Title with spaces", "https://example.com/path with spaces"),
        ("Title\nwith\nnewlines", "https://example.com"),
        ("Title\twith\ttabs", "https://example.com"),
        ("Title'with'quotes", "https://example.com"),
        ("Title\"with\"doublequotes", "https://example.com"),
    ];

    for (title, url) in special_chars {
        let result = copy_markdown_link_to_clipboard(title, url);
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(matches!(e, Error::Config(_)));
            }
        }
    }
}

#[test]
fn test_error_integration() {
    initialize();

    let result = copy_markdown_link_to_clipboard("Test", "https://example.com");
    match result {
        Ok(_) => {}
        Err(e) => {
            assert!(matches!(e, Error::Config(_)));
            assert!(
                e.to_string().contains("Failed to"),
                "Error should contain descriptive message"
            );
        }
    }
}