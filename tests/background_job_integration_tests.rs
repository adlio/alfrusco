mod common;

use std::fs;
use std::process::Command;
use std::time::Duration;

use common::{
    create_job_id, create_test_workflow_with_temp, find_job_directory, last_run_mtime,
    wait_for_job_completion, wait_for_job_status,
};

#[test]
fn test_background_job_lifecycle() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();

    // Run a simple command in the background
    let mut cmd = Command::new("echo");
    cmd.arg("test");

    // Run with a short max_age to ensure it's considered stale
    workflow.run_in_background("test_job", Duration::from_secs(0), cmd);

    // Verify that the job directory was created
    let job_dir = temp_path.join("jobs").join(create_job_id("test_job"));
    assert!(job_dir.exists());

    // Verify that a PID file was created
    let pid_file = job_dir.join("job.pid");
    assert!(pid_file.exists());

    wait_for_job_completion(100);

    // Create a status file with "success" to simulate a successful job
    let status_file = job_dir.join("job.status");
    fs::write(&status_file, "success").unwrap();

    // Run the job again - this should now create a last_run file
    let mut cmd2 = Command::new("echo");
    cmd2.arg("test2");
    workflow.run_in_background("test_job", Duration::from_secs(0), cmd2);

    wait_for_job_completion(500);

    // Verify that the last_run file exists
    let last_run_file = job_dir.join("job.last_run");
    assert!(last_run_file.exists());
}

#[test]
fn test_background_job_fresh_vs_stale() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();

    // Create the job directory and a last_run file manually
    let job_dir = temp_path.join("jobs").join("fresh_job");
    fs::create_dir_all(&job_dir).unwrap();
    let last_run_file = job_dir.join("job.last_run");
    fs::write(&last_run_file, "2023-01-01T00:00:00Z").unwrap();

    // Set the file's modified time to now
    let now = std::time::SystemTime::now();
    let file = fs::File::options()
        .write(true)
        .open(&last_run_file)
        .unwrap();
    let times = fs::FileTimes::new().set_accessed(now).set_modified(now);
    file.set_times(times).unwrap();

    // Run with a long max_age to ensure it's considered fresh
    let mut cmd = Command::new("echo");
    cmd.arg("test");
    workflow.run_in_background("fresh_job", Duration::from_hours(1), cmd);

    // Verify that no new PID file was created (job wasn't run)
    let pid_file = job_dir.join("job.pid");
    assert!(!pid_file.exists());
}
#[test]
fn test_background_job_success_status() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();

    // Run a command that will succeed
    let mut cmd = Command::new("echo");
    cmd.arg("success test");
    workflow.run_in_background("success_job", Duration::from_secs(0), cmd);

    wait_for_job_completion(500);

    // Find the job directory that was created
    let jobs_dir = temp_path.join("jobs");
    let job_dir = find_job_directory(&jobs_dir).expect("Should have found a job directory");

    // Verify that the status file shows "success"
    let status_file = job_dir.join("job.status");
    if status_file.exists() {
        let status = fs::read_to_string(&status_file).unwrap();
        assert_eq!(status.trim(), "success");
    }

    // Verify that the last_run file was created (since the job succeeded)
    let last_run_file = job_dir.join("job.last_run");
    assert!(last_run_file.exists());
}

#[test]
fn test_background_job_failure_status() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();

    // Create the job directory
    let job_dir = temp_path.join("jobs").join(create_job_id("failure_job"));
    fs::create_dir_all(&job_dir).unwrap();

    // Run a command that will fail
    let cmd = Command::new("false"); // Command that always fails
    workflow.run_in_background("failure_job", Duration::from_secs(0), cmd);

    wait_for_job_completion(500);

    // Verify that the status file shows "failed"
    let status_file = job_dir.join("job.status");
    if status_file.exists() {
        let status = fs::read_to_string(&status_file).unwrap();
        assert_eq!(status.trim(), "failed");
    }

    // Verify that the last_run file was created (even for failed jobs)
    let last_run_file = job_dir.join("job.last_run");
    assert!(
        last_run_file.exists(),
        "last_run file should exist even for failed jobs"
    );
}
#[test]
fn test_failed_jobs_are_retried() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();
    let jobs_dir = temp_path.join("jobs");
    let job_id = create_job_id("retry_test_job");
    let job_dir = jobs_dir.join(&job_id);
    let status_file = job_dir.join("job.status");

    // Step 1: Run a job that will fail
    let before_first = last_run_mtime(&job_dir);
    let cmd = Command::new("false");
    workflow.run_in_background("retry_test_job", Duration::from_secs(0), cmd);

    // Wait for the job to actually complete with "failed" status
    wait_for_job_status(&status_file, "failed", before_first, 5000)
        .expect("First job should complete with 'failed' status");

    // Step 2: Run the same job again - it should be re-run because it failed.
    // Wait past one coarse filesystem clock tick so the second run's last_run
    // timestamp cannot equal the first one's.
    std::thread::sleep(Duration::from_millis(50));
    let before_second = last_run_mtime(&job_dir);
    let cmd2 = Command::new("false");
    workflow.run_in_background("retry_test_job", Duration::from_secs(0), cmd2);

    // Wait for the second job to complete
    wait_for_job_status(&status_file, "failed", before_second, 5000)
        .expect("Second job should complete with 'failed' status");

    // Verify the job directory exists
    let job_dir = find_job_directory(&jobs_dir).expect("Should have found a job directory");

    // Verify that the last_run file exists (jobs should be tracked even when they fail)
    let last_run_file = job_dir.join("job.last_run");
    assert!(
        last_run_file.exists(),
        "last_run file should exist even for failed jobs"
    );

    // Verify status file shows "failed"
    let status = fs::read_to_string(&status_file).unwrap();
    assert_eq!(status.trim(), "failed");
}

#[test]
fn test_shell_escaping_in_background_jobs() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let temp_path = temp_dir.path();

    // Create a command with arguments that need proper escaping
    let mut cmd = Command::new("echo");
    cmd.arg("Hello World"); // Argument with spaces
    cmd.arg("CloudSmith"); // This was a specific case that caused issues

    // Run the command in the background
    workflow.run_in_background("quoted_args_job", Duration::from_secs(0), cmd);

    wait_for_job_completion(500);

    // Get the job directory and verify it was created successfully
    let jobs_dir = temp_path.join("jobs");
    assert!(jobs_dir.exists(), "Jobs directory should exist");

    // Find the job directory
    let job_dirs: Vec<_> = fs::read_dir(&jobs_dir).unwrap().collect();
    assert_eq!(job_dirs.len(), 1, "Should have exactly one job directory");

    let job_dir = job_dirs.into_iter().next().unwrap().unwrap().path();

    // Verify the job completed successfully (shell escaping worked)
    let status_file = job_dir.join("job.status");
    if status_file.exists() {
        let status = fs::read_to_string(&status_file).unwrap();
        assert_eq!(
            status.trim(),
            "success",
            "Job should have succeeded with proper shell escaping"
        );
    }
}

/// Stops a job's bash wrapper and the command it started.
fn kill_job(pid: &str) {
    let _ = Command::new("pkill").args(["-P", pid]).status();
    let _ = Command::new("kill").arg(pid).status();
}

#[test]
fn test_running_job_is_detected_and_not_started_twice() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let job_dir = temp_dir.path().join("jobs").join(create_job_id("long_job"));
    let pid_file = job_dir.join("job.pid");

    let mut cmd = Command::new("sleep");
    cmd.arg("5");
    workflow.run_in_background("long_job", Duration::from_secs(0), cmd);
    let first_pid = fs::read_to_string(&pid_file).unwrap();

    // The job is still running, so a second call must report it rather than
    // start another copy.
    let mut cmd2 = Command::new("sleep");
    cmd2.arg("5");
    workflow.run_in_background("long_job", Duration::from_secs(0), cmd2);
    let second_pid = fs::read_to_string(&pid_file).unwrap();
    let status = fs::read_to_string(job_dir.join("job.status")).unwrap();

    kill_job(first_pid.trim());

    assert_eq!(first_pid, second_pid, "a running job must not be restarted");
    assert_eq!(status.trim(), "running");

    let mut out = Vec::new();
    workflow.response.write(&mut out).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "one status item per call");
    let subtitle = items[1]["subtitle"].as_str().unwrap();
    assert!(
        subtitle.contains("running for"),
        "status item should describe the running job: {subtitle}"
    );
}

#[test]
fn test_stale_pid_file_for_exited_process_starts_new_job() {
    let (mut workflow, temp_dir) = create_test_workflow_with_temp();
    let job_dir = temp_dir
        .path()
        .join("jobs")
        .join(create_job_id("stale_pid_job"));
    fs::create_dir_all(&job_dir).unwrap();
    let pid_file = job_dir.join("job.pid");

    // Leave behind the PID of a process that has already exited and been reaped.
    let mut gone = Command::new("true").spawn().unwrap();
    let gone_pid = gone.id();
    gone.wait().unwrap();
    fs::write(&pid_file, gone_pid.to_string()).unwrap();
    fs::write(job_dir.join("job.status"), "running").unwrap();

    let mut cmd = Command::new("sleep");
    cmd.arg("5");
    workflow.run_in_background("stale_pid_job", Duration::from_secs(0), cmd);

    let new_pid = fs::read_to_string(&pid_file).unwrap();
    kill_job(new_pid.trim());
    assert_ne!(
        new_pid.trim(),
        gone_pid.to_string(),
        "a PID file for an exited process must not block a new run"
    );
}
