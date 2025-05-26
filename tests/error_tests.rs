use std::env::VarError;
use std::io;

use alfrusco::{Error, WorkflowError};

#[test]
fn test_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = Error::Io(io_error);

    assert!(matches!(error, Error::Io(_)));
    assert_eq!(error.to_string(), "IO Error: file not found");
}

#[test]
fn test_var_error() {
    let var_error = VarError::NotPresent;
    let error = Error::Var(var_error);

    assert!(matches!(error, Error::Var(_)));
    assert_eq!(
        error.to_string(),
        "Var Error: environment variable not found"
    );
}

#[test]
fn test_missing_env_var_error() {
    let error = Error::MissingEnvVar("TEST_VAR".to_string());

    assert!(matches!(error, Error::MissingEnvVar(_)));
    assert_eq!(error.to_string(), "Missing environment variable: TEST_VAR");
}

#[test]
fn test_workflow_error_from_string() {
    let error: Error = "Test error message".into();

    assert!(matches!(error, Error::Workflow(_)));
    assert_eq!(error.to_string(), "Workflow Error: Test error message");
}

#[test]
fn test_workflow_error_from_string_type() {
    let error: Error = String::from("Test error message").into();

    assert!(matches!(error, Error::Workflow(_)));
    assert_eq!(error.to_string(), "Workflow Error: Test error message");
}

#[test]
fn test_error_item_with_source() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = Error::Io(io_error);

    // Just verify that error_item() doesn't panic
    let _item = error.error_item();
}

#[test]
fn test_error_item_without_source() {
    let error = Error::MissingEnvVar("TEST_VAR".to_string());

    // Just verify that error_item() doesn't panic
    let _item = error.error_item();
}
