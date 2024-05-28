use anyhow::Result;
use std::env;

/// Retrives a string-formatted value from the Workflow configuration
pub fn get_string(key: String) -> Result<String> {
    env::var(key).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_string() {
        env::set_var("TEST_KEY", "test_value");
        assert_eq!(get_string("TEST_KEY".to_string()).unwrap(), "test_value");
    }
}
