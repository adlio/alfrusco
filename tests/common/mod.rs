use std::path::Path;

use alfrusco::config::WorkflowConfig;
use alfrusco::Workflow;
use tempfile::TempDir;

/// Creates a test workflow with a temporary directory
pub fn create_test_workflow(temp_path: &Path) -> Workflow {
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

/// Creates a test workflow with a new temporary directory
pub fn create_test_workflow_with_temp() -> (Workflow, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let workflow = create_test_workflow(temp_dir.path());
    (workflow, temp_dir)
}

/// Creates a filesystem-safe hash from a job name (same as in background_job.rs)
pub fn create_job_id(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{hash:x}")
}

/// Helper to wait for background job completion with timeout
#[allow(dead_code)]
pub fn wait_for_job_completion(max_wait_ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(max_wait_ms));
}

/// Helper to wait for a job to complete with a specific status.
/// This waits for:
/// 1. The status file to contain the expected value
/// 2. The last_run file to be modified after we started waiting (indicating script fully completed)
#[allow(dead_code)]
pub fn wait_for_job_status(
    status_file: &Path,
    expected_status: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    use std::time::SystemTime;

    let start_time = SystemTime::now();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    let job_dir = status_file.parent().unwrap();
    let last_run_file = job_dir.join("job.last_run");

    loop {
        // Check all conditions for job completion:
        // 1. Status file has expected value
        // 2. last_run file was modified after we started waiting (script completed)
        let status_ok = status_file.exists()
            && std::fs::read_to_string(status_file)
                .map(|s| s.trim() == expected_status)
                .unwrap_or(false);

        let last_run_recent = last_run_file.exists()
            && std::fs::metadata(&last_run_file)
                .and_then(|m| m.modified())
                .map(|mtime| mtime >= start_time)
                .unwrap_or(false);

        if status_ok && last_run_recent {
            // Give a tiny bit more time for the bash process to fully exit
            std::thread::sleep(std::time::Duration::from_millis(10));
            return Ok(());
        }

        if start.elapsed() > timeout {
            let actual_status = if status_file.exists() {
                std::fs::read_to_string(status_file).unwrap_or_else(|_| "<unreadable>".to_string())
            } else {
                "<file not found>".to_string()
            };
            return Err(format!(
                "Timeout waiting for job completion. Status: '{}' (expected '{}'), last_run recent: {}",
                actual_status.trim(),
                expected_status,
                last_run_recent
            ));
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Helper to find a job directory in the jobs folder
#[allow(dead_code)]
pub fn find_job_directory(jobs_dir: &Path) -> Option<std::path::PathBuf> {
    if !jobs_dir.exists() {
        return None;
    }

    for entry in std::fs::read_dir(jobs_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}
