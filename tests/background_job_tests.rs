use std::fs::{self, create_dir_all};
use std::path::{Path};
use std::process::Command;
use std::time::Duration;

use alfrusco::config::WorkflowConfig;
use alfrusco::Workflow;
use tempfile::TempDir;

#[test]
fn test_background_job_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Run a simple command in the background
    let mut cmd = Command::new("echo");
    cmd.arg("test");

    // Run with a short max_age to ensure it's considered stale
    workflow.run_in_background("test_job", Duration::from_secs(0), cmd);

    // Verify that the job directory was created
    let job_dir = temp_path.join("jobs").join("test_job");
    assert!(job_dir.exists());

    // Verify that a PID file was created
    let pid_file = job_dir.join("job.pid");
    assert!(pid_file.exists());

    // Sleep briefly to allow the process to complete
    std::thread::sleep(Duration::from_millis(100));

    // Run the job again - this should now create a last_run file
    let mut cmd2 = Command::new("echo");
    cmd2.arg("test2");
    workflow.run_in_background("test_job", Duration::from_secs(0), cmd2);

    // Verify that the last_run file exists
    let last_run_file = job_dir.join("job.last_run");
    assert!(last_run_file.exists());
}

#[test]
fn test_background_job_fresh() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Create the job directory and a last_run file manually
    let job_dir = temp_path.join("jobs").join("fresh_job");
    create_dir_all(&job_dir).unwrap();
    let last_run_file = job_dir.join("job.last_run");
    fs::write(&last_run_file, "2023-01-01T00:00:00Z").unwrap();

    // Set the file's modified time to now
    let now = std::time::SystemTime::now();
    let file = std::fs::File::options()
        .write(true)
        .open(&last_run_file)
        .unwrap();
    let times = std::fs::FileTimes::new()
        .set_accessed(now)
        .set_modified(now);
    file.set_times(times).unwrap();

    // Run with a long max_age to ensure it's considered fresh
    let mut cmd = Command::new("echo");
    cmd.arg("test");
    workflow.run_in_background("fresh_job", Duration::from_secs(3600), cmd);

    // Verify that no new PID file was created (job wasn't run)
    let pid_file = job_dir.join("job.pid");
    assert!(!pid_file.exists());
}

#[test]
fn test_background_job_error() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Create a command that will fail
    let cmd = Command::new("non_existent_command");

    // Run the command
    workflow.run_in_background("error_job", Duration::from_secs(0), cmd);

    // Verify that the job directory was created
    let job_dir = temp_path.join("jobs").join("error_job");
    assert!(job_dir.exists());
}

fn create_test_workflow(temp_path: &Path) -> Workflow {
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
        workflow_cache: temp_path.to_path_buf(),
        workflow_data: temp_path.to_path_buf(),
        workflow_name: "Test Workflow".to_string(),
        workflow_description: Some("Test workflow description".to_string()),
        workflow_version: Some("1.0".to_string()),
        workflow_uid: Some("test.workflow.123".to_string()),
        workflow_keyword: None,
        debug: true,
    };

    Workflow::new(config).expect("Failed to create workflow")
}
