use std::process::Command;
use std::time::Duration;

use alfrusco::{config, Workflow};
use alfrusco::config::{ConfigProvider, WorkflowConfig};
use tempfile::TempDir;

/// Test that simulates a command that fails to spawn
#[test]
fn test_background_job_command_spawn_error() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create a workflow with the temp directory as cache
    let config = WorkflowConfig {
        preferences: Some("/Users/Test/Alfred.alfredpreferences".to_string()),
        preferences_localhash: Some("test123".to_string()),
        theme: Some("alfred.theme.test".to_string()),
        theme_background: Some("rgba(255,255,255,0.98)".to_string()),
        theme_selection_background: Some("rgba(255,255,255,0.98)".to_string()),
        theme_subtext: Some("3".to_string()),
        version: "5.0".to_string(),
        version_build: "2058".to_string(),
        workflow_bundleid: "com.test.workflow".to_string(),
        workflow_cache: temp_path.clone(),
        workflow_data: temp_path.clone(),
        workflow_name: "Test Workflow".to_string(),
        workflow_description: Some("Test workflow description".to_string()),
        workflow_version: Some("1.0".to_string()),
        workflow_uid: Some("test.workflow.123".to_string()),
        workflow_keyword: None,
        debug: true,
    };
    
    let mut workflow = Workflow::new(config).expect("Failed to create workflow");
    
    // Create a command that will fail to spawn
    let cmd = Command::new("non_existent_command_that_definitely_does_not_exist");
    
    // Run the command in the background
    workflow.run_in_background("error_job", Duration::from_secs(0), cmd);
    
    // The job directory should have been created
    let job_dir = temp_path.join("jobs").join("error_job");
    assert!(job_dir.exists());
    
    // But no PID file should exist
    let pid_file = job_dir.join("job.pid");
    assert!(!pid_file.exists());
}

/// Test that simulates environment variable errors in config
#[test]
fn test_config_env_var_errors() {
    // Test each required environment variable
    let required_vars = [
        "alfred_workflow_bundleid",
        "alfred_workflow_cache",
        "alfred_workflow_data",
        "alfred_version",
        "alfred_version_build",
        "alfred_workflow_name",
    ];
    
    for var in required_vars {
        // Set up environment with all required vars except the one we're testing
        let mut env_vars = Vec::new();
        for &required_var in &required_vars {
            if required_var != var {
                env_vars.push((required_var, Some("/test/value")));
            } else {
                env_vars.push((required_var, None));
            }
        }
        
        temp_env::with_vars(env_vars, || {
            let provider = config::AlfredEnvProvider;
            let result = provider.config();
            
            // Verify that the result is an error
            assert!(result.is_err());
            
            // Verify that the error message mentions the missing variable
            let error = result.unwrap_err();
            assert!(error.to_string().contains(var));
        });
    }
}
