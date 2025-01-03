use std::path::PathBuf;

use crate::config::WorkflowConfig;
use crate::error::Result;
use crate::item::Item;
use crate::response::Response;

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

#[cfg(test)]
mod tests {
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
}
