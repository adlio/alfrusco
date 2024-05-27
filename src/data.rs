use std::env;
use std::fs;
use std::path::PathBuf;

pub const DATA_DIR_ENV_VAR: &str = "alfred_workflow_data";

pub fn data_dir() -> PathBuf {
    let data_dir = match env::var(DATA_DIR_ENV_VAR) {
        Ok(dir_str) => PathBuf::from(dir_str),
        Err(_) => env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("testdata")
            .join("workflowdata"),
    };
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    data_dir
}
