use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use alfrusco::{config, Workflow};
use alfrusco::config::{ConfigProvider, WorkflowConfig};
use tempfile::TempDir;

/// A test helper that creates a read-only directory to simulate permission errors
struct ReadOnlyDir {
    path: PathBuf,
}

impl ReadOnlyDir {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o555); // read + execute, no write
            fs::set_permissions(&path, perms).unwrap();
        }
        
        ReadOnlyDir { path }
    }
    
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReadOnlyDir {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Restore write permissions so the directory can be deleted
            let perms = fs::Permissions::from_mode(0o755);
            let _ = fs::set_permissions(&self.path, perms);
        }
        
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Test that simulates file system permission errors
#[test]
fn test_background_job_filesystem_error() {
    // Skip this test on Windows as permission handling is different
    #[cfg(not(windows))]
    {
        // Create a read-only directory
        let read_only_dir = ReadOnlyDir::new();
        
        // Create a workflow with the read-only directory as cache
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
            workflow_cache: read_only_dir.path().to_path_buf(),
            workflow_data: read_only_dir.path().to_path_buf(),
            workflow_name: "Test Workflow".to_string(),
            workflow_description: Some("Test workflow description".to_string()),
            workflow_version: Some("1.0".to_string()),
            workflow_uid: Some("test.workflow.123".to_string()),
            workflow_keyword: None,
            debug: true,
        };
        
        let mut workflow = Workflow::new(config).expect("Failed to create workflow");
        
        // Try to run a command in the background
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        
        // This should fail because we can't write to the jobs directory
        workflow.run_in_background("test_job", Duration::from_secs(0), cmd);
        
        // The job directory should not have been created
        let job_dir = read_only_dir.path().join("jobs").join("test_job");
        assert!(!job_dir.exists());
    }
}

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
    let mut cmd = Command::new("non_existent_command_that_definitely_does_not_exist");
    
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

/// Test that simulates a file system error when creating workflow directories
#[test]
fn test_workflow_directory_creation_error() {
    // Skip this test on Windows as permission handling is different
    #[cfg(not(windows))]
    {
        // Create a read-only directory
        let read_only_dir = ReadOnlyDir::new();
        
        // Create a workflow config with the read-only directory as parent
        let workflow_cache = read_only_dir.path().join("workflow_cache");
        let workflow_data = read_only_dir.path().join("workflow_data");
        
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
            workflow_cache,
            workflow_data,
            workflow_name: "Test Workflow".to_string(),
            workflow_description: Some("Test workflow description".to_string()),
            workflow_version: Some("1.0".to_string()),
            workflow_uid: Some("test.workflow.123".to_string()),
            workflow_keyword: None,
            debug: true,
        };
        
        // Creating the workflow should fail because we can't create the directories
        let result = Workflow::new(config);
        assert!(result.is_err());
    }
}
