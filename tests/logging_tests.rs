use std::time::Duration;
use std::{fs, thread};

#[test]
fn test_init_logging() {
    use alfrusco::config::TestingProvider;
    use tempfile::TempDir;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    // Initialize logging
    let result = alfrusco::init_logging(&provider);

    // First initialization should succeed
    assert!(result.is_ok());

    // Check that the log file was created
    let log_file = temp_dir.path().join("workflow_cache").join("workflow.log");

    // Log some messages
    log::info!("Test info message");
    log::debug!("Test debug message");
    log::warn!("Test warning message");

    // Give the logger a moment to write to the file
    thread::sleep(Duration::from_millis(100));

    // Check that the log file exists
    assert!(log_file.exists());

    // Read the log file content
    let log_content = fs::read_to_string(&log_file).unwrap();

    // Verify log messages were written
    assert!(
        log_content.contains("Test info message")
            || log_content.contains("Test debug message")
            || log_content.contains("Test warning message")
    );

    // Second initialization should fail (logger already initialized)
    let second_result = alfrusco::init_logging(&provider);
    assert!(second_result.is_err());
}
