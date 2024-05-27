use std::env;
use std::fs;
use std::path::PathBuf;

pub const CACHE_DIR_ENV_VAR: &str = "alfred_workflow_cache";

pub fn cache_dir() -> PathBuf {
    let cache_dir = match env::var(CACHE_DIR_ENV_VAR) {
        Ok(dir_str) => PathBuf::from(dir_str),
        Err(_) => env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("testdata")
            .join("workflowcache"),
    };
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
    cache_dir
}
