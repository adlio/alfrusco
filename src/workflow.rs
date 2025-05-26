use std::path::PathBuf;

use crate::config::{ConfigProvider, WorkflowConfig};
use crate::error::Result;
use crate::internal_handlers::handle;
use crate::item::Item;
use crate::response::Response;
use crate::sort_and_filter::filter_and_sort_items;

/// Workflow represents an active execution of an Alfred workflow.
///
/// It maintains the state of the current Response, and owns the Workflow
/// configuration information (cache and data directories, versions,
/// workflow names, etc).  This struct is instantiated automatically as
/// part of the alfrusco::execute_* process, so alfrusco consumers needn't
/// create this struct from scratch.
///
#[derive(Debug)]
pub struct Workflow {
    pub config: WorkflowConfig,
    pub response: Response,

    pub keyword: Option<String>,
    pub(crate) sort_and_filter_results: bool,
}

impl Workflow {
    pub fn new(config: WorkflowConfig) -> Result<Self> {
        // Ensure workflow data and cache directories exist
        std::fs::create_dir_all(&config.workflow_data)?;
        std::fs::create_dir_all(&config.workflow_cache)?;

        Ok(Workflow {
            config,
            response: Response::default(),
            keyword: None,
            sort_and_filter_results: false,
        })
    }

    pub fn set_filter_keyword(&mut self, keyword: String) {
        self.keyword = Some(keyword);
        self.sort_and_filter_results = true;
    }

    pub fn items(&mut self, items: Vec<Item>) {
        self.response.items(items);
    }

    pub fn prepend_item(&mut self, item: Item) {
        self.response.prepend_items(vec![item]);
    }

    pub fn prepend_items(&mut self, items: Vec<Item>) {
        self.response.prepend_items(items);
    }

    pub fn append_items(&mut self, items: Vec<Item>) {
        self.response.append_items(items);
    }

    pub fn append_item(&mut self, item: Item) {
        self.response.append_items(vec![item]);
    }

    pub fn skip_knowledge(&mut self, skip: bool) {
        self.response.skip_knowledge(skip);
    }

    pub fn data_dir(&self) -> PathBuf {
        self.config.workflow_data.clone()
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.config.workflow_cache.clone()
    }
}

/// Sets up a workflow using the provided configuration provider.
///
/// This function:
/// 1. Loads configuration from the provider
/// 2. Creates a new workflow instance
/// 3. Handles special commands (clipboard operations, workflow directories)
///
/// # Panics
///
/// This function will panic if:
/// - The configuration cannot be loaded
/// - The workflow cannot be created
pub fn setup_workflow(provider: &dyn ConfigProvider) -> Workflow {
    let config = provider.config();
    if config.is_err() {
        eprintln!("Error loading config: {}", config.unwrap_err());
        std::process::exit(1);
    }

    let mut workflow = match Workflow::new(config.unwrap()) {
        Ok(workflow) => workflow,
        Err(e) => {
            eprintln!("Error creating workflow: {e}");
            std::process::exit(1);
        }
    };

    // Handle special commands after creating the workflow
    if handle(&mut workflow) {
        std::process::exit(0);
    }

    workflow
}

/// Finalizes a workflow by applying filtering if needed and writing the response.
///
/// This function:
/// 1. Applies filtering and sorting if enabled
/// 2. Writes the response to the provided writer
///
/// # Panics
///
/// This function will panic if the response cannot be written to the writer.
pub fn finalize_workflow(mut workflow: Workflow, writer: &mut dyn std::io::Write) {
    if workflow.sort_and_filter_results {
        if let Some(keyword) = workflow.keyword.clone() {
            workflow.response.items = filter_and_sort_items(workflow.response.items, keyword);
        }
    }
    match workflow.response.write(writer) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing response: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::TempDir;

    use super::*;
    use crate::config::{self, ConfigProvider};

    fn test_workflow() -> (Workflow, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let config = config::TestingProvider(dir.path().into()).config().unwrap();
        (Workflow::new(config).unwrap(), dir)
    }

    #[test]
    fn test_new_workflow() {
        let (workflow, _dir) = test_workflow();
        assert_eq!(workflow.response.items.len(), 0);
        assert_eq!(workflow.keyword, None);
        assert!(!workflow.sort_and_filter_results);
    }

    #[test]
    fn test_prepend_item() {
        let (mut workflow, _dir) = test_workflow();
        let initial_item = Item::new("Initial Item");
        workflow.items(vec![initial_item]);

        let prepended_item = Item::new("Prepended Item");
        workflow.prepend_item(prepended_item);

        assert_eq!(workflow.response.items.len(), 2);
        assert_eq!(workflow.response.items[0].title, "Prepended Item");
        assert_eq!(workflow.response.items[1].title, "Initial Item");
    }

    #[test]
    fn test_prepend_items() {
        let (mut workflow, _dir) = test_workflow();
        workflow.items(vec![
            Item::new("First Item"),
            Item::new("Second Item"),
            Item::new("Third Item"),
        ]);

        let prepended_items = vec![
            Item::new("Prepended Item 1"),
            Item::new("Prepended Item 2"),
            Item::new("Prepended Item 3"),
        ];

        workflow.prepend_items(prepended_items);

        assert_eq!(workflow.response.items.len(), 6);
        assert_eq!(workflow.response.items[0].title, "Prepended Item 1");
        assert_eq!(workflow.response.items[1].title, "Prepended Item 2");
        assert_eq!(workflow.response.items[3].title, "First Item");
        assert_eq!(workflow.response.items[5].title, "Third Item");
    }

    #[test]
    fn test_append_item() {
        let (mut workflow, _dir) = test_workflow();
        let initial_item = Item::new("Initial Item");
        workflow.items(vec![initial_item]);

        let appended_item = Item::new("Appended Item");
        workflow.append_item(appended_item);

        assert_eq!(workflow.response.items.len(), 2);
        assert_eq!(workflow.response.items[0].title, "Initial Item");
        assert_eq!(workflow.response.items[1].title, "Appended Item");
    }

    #[test]
    fn test_append_items() {
        let (mut workflow, _dir) = test_workflow();
        workflow.items(vec![
            Item::new("First Item"),
            Item::new("Second Item"),
            Item::new("Third Item"),
        ]);

        let appended_items = vec![
            Item::new("Appended Item 1"),
            Item::new("Appended Item 2"),
            Item::new("Appended Item 3"),
        ];

        workflow.append_items(appended_items);

        assert_eq!(workflow.response.items.len(), 6);
        assert_eq!(workflow.response.items[0].title, "First Item");
        assert_eq!(workflow.response.items[3].title, "Appended Item 1");
        assert_eq!(workflow.response.items[5].title, "Appended Item 3");
    }

    #[test]
    fn test_finalize_workflow_with_filtering() {
        let (mut workflow, _dir) = test_workflow();

        // Add some items
        workflow.items(vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Fruit"),
            Item::new("Carrot").subtitle("Vegetable"),
        ]);

        // Enable filtering
        workflow.set_filter_keyword("fruit".to_string());

        // Create a buffer to capture the output
        let mut buffer = Cursor::new(Vec::new());

        // Finalize the workflow
        finalize_workflow(workflow, &mut buffer);

        // Get the output as a string
        let output = String::from_utf8(buffer.into_inner()).unwrap();

        // Verify filtering was applied (only fruit items should be included)
        assert!(output.contains("Apple"));
        assert!(output.contains("Banana"));
        assert!(!output.contains("Carrot"));
    }

    #[test]
    fn test_finalize_workflow_without_filtering() {
        let (mut workflow, _dir) = test_workflow();

        // Add some items
        workflow.items(vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Fruit"),
            Item::new("Carrot").subtitle("Vegetable"),
        ]);

        // Don't enable filtering (sort_and_filter_results remains false)

        // Create a buffer to capture the output
        let mut buffer = Cursor::new(Vec::new());

        // Finalize the workflow
        finalize_workflow(workflow, &mut buffer);

        // Get the output as a string
        let output = String::from_utf8(buffer.into_inner()).unwrap();

        // Verify all items are included (no filtering)
        assert!(output.contains("Apple"));
        assert!(output.contains("Banana"));
        assert!(output.contains("Carrot"));
    }

    #[test]
    fn test_setup_workflow() {
        // Create a test config provider
        let dir = tempfile::tempdir().unwrap().keep();
        let provider = config::TestingProvider(dir);

        // Set up the workflow
        let workflow = setup_workflow(&provider);

        // Verify the workflow was created correctly
        assert_eq!(workflow.response.items.len(), 0);
        assert_eq!(workflow.keyword, None);
        assert!(!workflow.sort_and_filter_results);

        // Verify the directories were created
        assert!(provider.0.join("workflow_data").exists());
        assert!(provider.0.join("workflow_cache").exists());
    }
}
