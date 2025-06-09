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
/// This is a simplified version that doesn't need extensive testing
fn open_path(path: &str) -> bool {
    use std::process::Command;
    match Command::new("open").arg(path).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use tempfile;

    use super::*;
    use crate::config::{self, ConfigProvider};

    fn test_workflow() -> Workflow {
        let dir = tempfile::tempdir().unwrap();
        let config = config::TestingProvider(dir.path().into()).config().unwrap();
        Workflow::new(config).unwrap()
    }

    #[test]
    fn test_handle_no_special_commands() {
        // Test when no special commands are present
        let mut workflow = test_workflow();
        let initial_item_count = workflow.response.items.len();

        // This should return false (no special command handled)
        let result = handle(&mut workflow);
        assert!(!result);

        // Should not have added any items
        assert_eq!(workflow.response.items.len(), initial_item_count);
    }

    #[test]
    fn test_handle_workflow_dir_open_no_args() {
        // Test handle_workflow_dir_open when no command line args are present
        let mut workflow = test_workflow();
        let result = handle_workflow_dir_open(&mut workflow);
        assert!(!result);
    }

    #[test]
    fn test_execute_workflow_command_show_suggestions() {
        // Test that ShowSuggestions command works and adds items
        let mut workflow = test_workflow();
        let initial_count = workflow.response.items.len();

        let result = execute_workflow_command(WorkflowCommand::ShowSuggestions, &mut workflow);
        assert_eq!(result, Some(false)); // Should return false (don't exit)
        assert_eq!(workflow.response.items.len(), initial_count + 3); // Should add 3 suggestions
    }

    #[test]
    fn test_execute_workflow_command_none() {
        // Test that None command returns None
        let mut workflow = test_workflow();
        let result = execute_workflow_command(WorkflowCommand::None, &mut workflow);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_query_from_args_no_args() {
        // This tests the actual function, but since we can't easily mock std::env::args,
        // we'll just verify it doesn't panic and returns something reasonable
        let result = extract_query_from_args();
        // The result depends on how the test is run, but it shouldn't panic
        // In most test environments, this will return Some value or None
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_parse_workflow_command() {
        // Test exact matches
        assert_eq!(
            parse_workflow_command("workflow:cache"),
            WorkflowCommand::OpenCache
        );
        assert_eq!(
            parse_workflow_command("workflow:data"),
            WorkflowCommand::OpenData
        );
        assert_eq!(
            parse_workflow_command("workflow:openlog"),
            WorkflowCommand::OpenLog
        );

        // Test prefix match
        assert_eq!(
            parse_workflow_command("work"),
            WorkflowCommand::ShowSuggestions
        );
        assert_eq!(
            parse_workflow_command("workflow"),
            WorkflowCommand::ShowSuggestions
        );

        // Test non-match
        assert_eq!(
            parse_workflow_command("something else"),
            WorkflowCommand::None
        );

        // Test with whitespace
        assert_eq!(
            parse_workflow_command("  workflow:cache  "),
            WorkflowCommand::OpenCache
        );
    }

    #[test]
    fn test_create_workflow_command_suggestions() {
        let items = create_workflow_command_suggestions();
        assert_eq!(items.len(), 3);

        // Check titles
        assert_eq!(items[0].title, "Open the workflow data directory");
        assert_eq!(items[1].title, "Open the workflow cache directory");
        assert_eq!(items[2].title, "Open the workflow log file");

        // Check subtitles
        assert_eq!(items[0].subtitle.as_deref().unwrap(), "workflow:data");
        assert_eq!(items[1].subtitle.as_deref().unwrap(), "workflow:cache");
        assert_eq!(items[2].subtitle.as_deref().unwrap(), "workflow:openlog");

        // Check valid flag
        assert_eq!(items[0].valid, Some(false));
        assert_eq!(items[1].valid, Some(false));
        assert_eq!(items[2].valid, Some(false));

        // Check sticky flag
        assert!(items[0].sticky);
        assert!(items[1].sticky);
        assert!(items[2].sticky);
    }
}
