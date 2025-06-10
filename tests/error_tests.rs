use alfrusco::{Error, WorkflowError};

#[test]
fn test_io_error() {
    let err = Error::Io(std::io::Error::other("test error"));
    assert!(err.to_string().contains("IO Error"));
}

#[test]
fn test_var_error() {
    let err = Error::Var(std::env::VarError::NotPresent);
    assert!(err.to_string().contains("Var Error"));
}

#[test]
fn test_missing_env_var_error() {
    let err = Error::MissingEnvVar("TEST_VAR".to_string());
    assert!(err.to_string().contains("Missing environment variable"));
}

#[test]
fn test_error_item_with_source() {
    let err = Error::Io(std::io::Error::other("test error"));
    let item = err.error_item();
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("Error:"));
    assert!(json.contains("subtitle"));
}

#[test]
fn test_error_item_without_source() {
    let err = Error::Workflow("test error".to_string());
    let item = err.error_item();
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("error occurred"));
}

#[test]
fn test_workflow_error_from_string() {
    let err: Error = "test error".to_string().into();
    assert!(matches!(err, Error::Workflow(_)));
}

#[test]
fn test_workflow_error_from_string_type() {
    let err: Error = "test error".into();
    assert!(matches!(err, Error::Workflow(_)));
}