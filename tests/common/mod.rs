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
    format!("{:x}", hash)
}

/// Helper to wait for background job completion with timeout
pub fn wait_for_job_completion(max_wait_ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(max_wait_ms));
}

/// Helper to find a job directory in the jobs folder
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
