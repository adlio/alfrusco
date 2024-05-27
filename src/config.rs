use anyhow::Result;
use std::env;

/// Retrives a string-formatted value from the Workflow configuration
pub fn get_string(key: String) -> Result<String> {
    env::var(key).map_err(|e| e.into())
}
