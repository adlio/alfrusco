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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_default_cache_dir() {
        env::remove_var(CACHE_DIR_ENV_VAR);
        let cache_dir = cache_dir();
        assert!(cache_dir.exists());
        assert_eq!(
            cache_dir,
            env::current_dir()
                .unwrap()
                .join("testdata")
                .join("workflowcache")
        );
    }

    #[test]
    fn test_custom_cache_dir() {
        let temp_dir = tempdir().unwrap();
        let td_str = temp_dir.path().to_str().unwrap().to_string();
        env::set_var(CACHE_DIR_ENV_VAR, td_str);

        let cache_dir = cache_dir();
        assert!(cache_dir.exists());
        assert_eq!(cache_dir, temp_dir.path());
    }
}
