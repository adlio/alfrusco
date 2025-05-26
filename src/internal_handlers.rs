use std::env::{self, var};
use std::path::PathBuf;
use std::process::Command;

use log::{debug, info};

use crate::clipboard::handle_clipboard;
use crate::{Item, Workflow};

/// Handle special commands based on environment variables.
/// This is the main entry point for special command handling.
///
/// Returns true if a command was handled and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle(workflow: &mut Workflow) -> bool {
    debug!("handle() called - checking for special commands");
    let result = handle_alfrusco_commands(workflow);

    // If the operation was successful and requires exiting, return true
    if result {
        debug!("Special command handled, should exit");
        return true;
    }

    debug!("No special commands handled, continuing workflow execution");
    false
}

/// Orchestrates handling of all alfrusco special commands without calling exit().
/// Returns true if a command was handled and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle_alfrusco_commands(workflow: &mut Workflow) -> bool {
    if handle_workflow_dir_open(workflow) {
        debug!("Workflow directory command handled");
        return true;
    }

    if handle_clipboard() {
        debug!("Clipboard command handled");
        return true;
    }

    // No special command was handled
    debug!("No special commands matched");
    false
}

/// Creates workflow command suggestion items
fn create_workflow_command_suggestions() -> Vec<Item> {
    vec![
        Item::new("Open the workflow data directory")
            .subtitle("workflow:data")
            .autocomplete("workflow:data")
            .valid(false)
            .sticky(true),
        Item::new("Open the workflow cache directory")
            .subtitle("workflow:cache")
            .autocomplete("workflow:cache")
            .valid(false)
            .sticky(true),
        Item::new("Open the workflow log file")
            .subtitle("workflow:openlog")
            .autocomplete("workflow:openlog")
            .valid(false)
            .sticky(true),
    ]
}

/// Handles opening workflow directories based on special queries.
/// Returns true if a workflow directory command was handled and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle_workflow_dir_open(workflow: &mut Workflow) -> bool {
    // First check command line arguments (like Python Alfred-Workflow does)
    let args: Vec<String> = env::args().collect();

    // Check for magic arguments in command line
    for arg in &args {
        let trimmed_arg = arg.trim();

        if trimmed_arg == "workflow:cache" {
            debug!("'workflow:cache' command detected in args");
            if let Ok(cache_dir) = var("alfred_workflow_cache") {
                debug!("Opening cache directory: {cache_dir}");
                open_directory(&cache_dir);
                return true;
            } else {
                debug!("alfred_workflow_cache environment variable not found");
            }
        } else if trimmed_arg == "workflow:data" {
            debug!("'workflow:data' command detected in args");
            if let Ok(data_dir) = var("alfred_workflow_data") {
                debug!("Opening data directory: {data_dir}");
                open_directory(&data_dir);
                return true;
            } else {
                debug!("alfred_workflow_data environment variable not found");
            }
        } else if trimmed_arg == "workflow:openlog" {
            debug!("'workflow:openlog' command detected in args");
            if let Ok(log_file) = get_log_file_path() {
                debug!("Opening log file: {log_file}");
                open_file(&log_file);
                return true;
            } else {
                debug!("Could not determine log file path");
            }
        } else if trimmed_arg.starts_with("work") {
            debug!("Command line arg '{trimmed_arg}' starts with 'work', showing workflow command suggestions");
            // Add workflow command suggestions
            workflow.append_items(create_workflow_command_suggestions());

            debug!(
                "Added workflow command suggestions to workflow response. Current item count: {}",
                workflow.response.items.len()
            );

            // Return false to indicate that we should continue with normal workflow execution
            // This allows our suggestions to appear alongside other workflow items
            return false;
        }
    }
    
    // Now check the alfred_query environment variable (like Python Alfred-Workflow does)
    if let Ok(query) = var("alfred_query") {
        let trimmed_query = query.trim();
        
        if trimmed_query == "workflow:cache" {
            debug!("'workflow:cache' command detected in alfred_query");
            if let Ok(cache_dir) = var("alfred_workflow_cache") {
                debug!("Opening cache directory: {cache_dir}");
                open_directory(&cache_dir);
                return true;
            } else {
                debug!("alfred_workflow_cache environment variable not found");
            }
        } else if trimmed_query == "workflow:data" {
            debug!("'workflow:data' command detected in alfred_query");
            if let Ok(data_dir) = var("alfred_workflow_data") {
                debug!("Opening data directory: {data_dir}");
                open_directory(&data_dir);
                return true;
            } else {
                debug!("alfred_workflow_data environment variable not found");
            }
        } else if trimmed_query == "workflow:openlog" {
            debug!("'workflow:openlog' command detected in alfred_query");
            if let Ok(log_file) = get_log_file_path() {
                debug!("Opening log file: {log_file}");
                open_file(&log_file);
                return true;
            } else {
                debug!("Could not determine log file path");
            }
        } else if trimmed_query.starts_with("work") {
            debug!("Query '{trimmed_query}' starts with 'work', showing workflow command suggestions");
            // Add workflow command suggestions
            workflow.append_items(create_workflow_command_suggestions());

            debug!(
                "Added workflow command suggestions to workflow response. Current item count: {}",
                workflow.response.items.len()
            );

            // Return false to indicate that we should continue with normal workflow execution
            // This allows our suggestions to appear alongside other workflow items
            return false;
        }
    }
    
    false
}

/// Open a directory in Finder
fn open_directory(path: &str) {
    info!("Opening directory: {path}");

    debug!("Executing command: open {path}");
    let output = Command::new("open")
        .arg(path)
        .output()
        .expect("Failed to execute open command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to open directory: {stderr}");
        debug!("open command failed with stderr: {stderr}");
    } else {
        debug!("open command executed successfully");
    }
}

/// Open a file with the default application
fn open_file(path: &str) {
    info!("Opening file: {path}");

    debug!("Executing command: open {path}");
    let output = Command::new("open")
        .arg(path)
        .output()
        .expect("Failed to execute open command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to open file: {stderr}");
        debug!("open command failed with stderr: {stderr}");
    } else {
        debug!("open command executed successfully");
    }
}

/// Get the path to the workflow log file
fn get_log_file_path() -> Result<String, &'static str> {
    // First try to get the log file from the environment variable
    if let Ok(log_file) = var("alfred_workflow_log") {
        debug!("Using log file from alfred_workflow_log: {log_file}");
        return Ok(log_file);
    }
    
    // If not set directly, construct the path from the cache directory and bundle ID
    if let (Ok(cache_dir), Ok(bundle_id)) = (var("alfred_workflow_cache"), var("alfred_workflow_bundleid")) {
        let mut log_path = PathBuf::from(cache_dir);
        log_path.push(format!("{}.log", bundle_id));
        
        let log_file = log_path.to_string_lossy().to_string();
        debug!("Constructed log file path: {log_file}");
        return Ok(log_file);
    }
    
    // Fallback to debug.log in cache directory if bundle ID is not available
    if let Ok(cache_dir) = var("alfred_workflow_cache") {
        let mut log_path = PathBuf::from(cache_dir);
        log_path.push("debug.log");
        
        let log_file = log_path.to_string_lossy().to_string();
        debug!("Constructed fallback log file path: {log_file}");
        return Ok(log_file);
    }
    
    Err("Could not determine log file path")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::Once;

    use temp_env::with_vars;

    use super::*;
    use crate::config::{self, ConfigProvider};

    // Initialize test environment
    static INIT: Once = Once::new();
    fn initialize() {
        INIT.call_once(|| {
            env::set_var("RUST_LOG", "debug");
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }

    fn test_workflow() -> Workflow {
        let dir = tempfile::tempdir().unwrap();
        let config = config::TestingProvider(dir.path().into()).config().unwrap();
        Workflow::new(config).unwrap()
    }

    #[test]
    fn test_handle_alfrusco_commands_no_command() {
        initialize();
        with_vars(
            [
                ("ALFRUSCO_COMMAND", None),
                ("TITLE", Some("Test Title")),
                ("URL", Some("https://example.com")),
                ("alfred_query", None),
            ],
            || {
                let mut workflow = test_workflow();
                let result = handle_alfrusco_commands(&mut workflow);
                assert!(!result);
            },
        );
    }

    #[test]
    fn test_handle_workflow_dir_open_cache() {
        initialize();
        with_vars(
            [
                ("alfred_query", Some("workflow:cache")),
                ("alfred_workflow_cache", Some("/test/cache/dir")),
            ],
            || {
                let mut workflow = test_workflow();
                // We can't fully test the directory opening in an automated test,
                // but we can verify that the function returns the expected result
                let result = handle_workflow_dir_open(&mut workflow);
                assert!(result);
            },
        );
    }

    #[test]
    fn test_handle_workflow_dir_open_data() {
        initialize();
        with_vars(
            [
                ("alfred_query", Some("workflow:data")),
                ("alfred_workflow_data", Some("/test/data/dir")),
            ],
            || {
                let mut workflow = test_workflow();
                // We can't fully test the directory opening in an automated test,
                // but we can verify that the function returns the expected result
                let result = handle_workflow_dir_open(&mut workflow);
                assert!(result);
            },
        );
    }

    #[test]
    fn test_handle_workflow_dir_open_missing_dir() {
        initialize();
        with_vars(
            [
                ("alfred_query", Some("workflow:cache")),
                ("alfred_workflow_cache", None::<&str>),
            ],
            || {
                let mut workflow = test_workflow();
                let result = handle_workflow_dir_open(&mut workflow);
                assert!(!result);
            },
        );

        with_vars(
            [
                ("alfred_query", Some("workflow:data")),
                ("alfred_workflow_data", None::<&str>),
            ],
            || {
                let mut workflow = test_workflow();
                let result = handle_workflow_dir_open(&mut workflow);
                assert!(!result);
            },
        );
    }

    #[test]
    fn test_handle_alfrusco_commands_workflow_cache() {
        initialize();
        with_vars(
            [
                ("alfred_query", Some("workflow:cache")),
                ("alfred_workflow_cache", Some("/test/cache/dir")),
            ],
            || {
                let mut workflow = test_workflow();
                // We can't fully test the directory opening in an automated test,
                // but we can verify that the function returns the expected result
                let result = handle_alfrusco_commands(&mut workflow);
                assert!(result);
            },
        );
    }

    #[test]
    fn test_handle_workflow_dir_open_openlog() {
        initialize();
        with_vars(
            [
                ("alfred_query", Some("workflow:openlog")),
                ("alfred_workflow_cache", Some("/test/cache/dir")),
            ],
            || {
                let mut workflow = test_workflow();
                // We can't fully test the file opening in an automated test,
                // but we can verify that the function returns the expected result
                let result = handle_workflow_dir_open(&mut workflow);
                assert!(result);
            },
        );
    }

    #[test]
    fn test_get_log_file_path() {
        initialize();
        
        // Test with alfred_workflow_log set
        with_vars(
            [
                ("alfred_workflow_log", Some("/test/log/file.log")),
            ],
            || {
                let result = get_log_file_path();
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), "/test/log/file.log");
            },
        );
        
        // Test with cache_dir and bundle_id set
        with_vars(
            [
                ("alfred_workflow_log", None::<&str>),
                ("alfred_workflow_cache", Some("/test/cache/dir")),
                ("alfred_workflow_bundleid", Some("com.example.myworkflow")),
            ],
            || {
                let result = get_log_file_path();
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), "/test/cache/dir/com.example.myworkflow.log");
            },
        );
        
        // Test fallback to debug.log when bundle_id is not available
        with_vars(
            [
                ("alfred_workflow_log", None::<&str>),
                ("alfred_workflow_cache", Some("/test/cache/dir")),
                ("alfred_workflow_bundleid", None::<&str>),
            ],
            || {
                let result = get_log_file_path();
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), "/test/cache/dir/debug.log");
            },
        );
        
        // Test with neither variable set
        with_vars(
            [
                ("alfred_workflow_log", None::<&str>),
                ("alfred_workflow_cache", None::<&str>),
            ],
            || {
                let result = get_log_file_path();
                assert!(result.is_err());
            },
        );
    }
    
    #[test]
    fn test_handle_workflow_suggestions() {
        initialize();
        with_vars([("alfred_query", Some("work"))], || {
            let mut workflow = test_workflow();
            let result = handle_workflow_dir_open(&mut workflow);

            // Should return false (continue execution) and add items to the workflow
            assert!(!result);
            assert_eq!(workflow.response.items.len(), 3);
            
            // Check that all three workflow commands are suggested
            let titles: Vec<&str> = workflow.response.items.iter()
                .map(|item| item.subtitle.as_deref().unwrap_or(""))
                .collect();
            
            assert!(titles.contains(&"workflow:data"));
            assert!(titles.contains(&"workflow:cache"));
            assert!(titles.contains(&"workflow:openlog"));
        });
    }
}
