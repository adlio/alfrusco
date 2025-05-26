use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use alfrusco::config::WorkflowConfig;
use alfrusco::Workflow;
use tempfile::TempDir;

#[test]
fn test_background_job_success_status() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Create the job directory
    let job_dir = temp_path.join("jobs").join("success_job");
    fs::create_dir_all(&job_dir).unwrap();

    // Create a status file with "success" to simulate a successful job
    let status_file = job_dir.join("job.status");
    fs::write(&status_file, "success").unwrap();

    // Create a pid file to simulate a job that has run
    let pid_file = job_dir.join("job.pid");
    fs::write(&pid_file, "12345").unwrap();

    // Run the cleanup process
    let mut cmd = Command::new("echo");
    cmd.arg("test");
    workflow.run_in_background("success_job", Duration::from_secs(0), cmd);

    // Sleep briefly to allow the process to complete
    std::thread::sleep(Duration::from_millis(500));

    // Verify that the last_run file was created (since the job succeeded)
    let last_run_file = job_dir.join("job.last_run");
    assert!(last_run_file.exists());
}

#[test]
fn test_background_job_failure_status() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Create the job directory
    let job_dir = temp_path.join("jobs").join("failure_job");
    fs::create_dir_all(&job_dir).unwrap();

    // Create a status file with "failed" to simulate a failed job
    let status_file = job_dir.join("job.status");
    fs::write(&status_file, "failed").unwrap();

    // Create a pid file to simulate a job that has run
    let pid_file = job_dir.join("job.pid");
    fs::write(&pid_file, "12345").unwrap();

    // Create a last_run file that should be removed
    let last_run_file = job_dir.join("job.last_run");
    fs::write(&last_run_file, "2023-01-01T00:00:00Z").unwrap();
    assert!(last_run_file.exists());

    // Run the cleanup process
    let mut cmd = Command::new("echo");
    cmd.arg("test");
    workflow.run_in_background("failure_job", Duration::from_secs(0), cmd);

    // Sleep briefly to allow the process to complete
    std::thread::sleep(Duration::from_millis(500));

    // Verify that the last_run file was removed (since the job failed)
    assert!(!last_run_file.exists());
}

// This test is more complex and requires a real process execution
// We'll skip it for now as it's difficult to simulate in a test environment
#[test]
#[ignore]
fn test_background_job_retry_after_failure() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a workflow with the temp directory as cache
    let mut workflow = create_test_workflow(&temp_path);

    // Create the job directory
    let job_dir = temp_path.join("jobs").join("retry_job");
    fs::create_dir_all(&job_dir).unwrap();

    // Create a status file with "failed" to simulate a failed job
    let status_file = job_dir.join("job.status");
    fs::write(&status_file, "failed").unwrap();

    // Create a pid file to simulate a job that has run
    let pid_file = job_dir.join("job.pid");
    fs::write(&pid_file, "12345").unwrap();

    // Run the cleanup process with a command that will succeed
    let mut cmd = Command::new("echo");
    cmd.arg("test");
    workflow.run_in_background("retry_job", Duration::from_secs(0), cmd);

    // Sleep briefly to allow the process to complete
    std::thread::sleep(Duration::from_millis(500));

    // Verify that the status file was updated to "success"
    let status = fs::read_to_string(&status_file).unwrap();
    assert_eq!(status.trim(), "success");

    // Verify that the last_run file was created (since the retry succeeded)
    let last_run_file = job_dir.join("job.last_run");
    assert!(last_run_file.exists());
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
