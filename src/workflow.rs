use crate::WorkflowConfig;
use async_trait::async_trait;

use crate::error::WorkflowError;
use crate::{filter_and_sort_items, Item, Response};

pub trait Runnable {
    type Error: WorkflowError;
    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait AsyncRunnable {
    type Error: WorkflowError;
    async fn run_async(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}

pub struct Workflow {
    pub config: WorkflowConfig,

    pub keyword: Option<String>,
    pub sort_and_filter_results: bool,

    pub response: Response,
}

impl Workflow {
    pub fn new(config: WorkflowConfig) -> Self {
        Workflow {
            config,
            keyword: None,
            sort_and_filter_results: false,
            response: Response::default(),
        }
    }

    pub fn execute<R: Runnable>(
        config: WorkflowConfig,
        runnable: R,
        writer: &mut dyn std::io::Write,
    ) {
        let mut workflow = Workflow::new(config);
        match runnable.run(&mut workflow) {
            Ok(_) => {}
            Err(e) => {
                workflow.prepend_item(e.error_item());
            }
        }

        if workflow.sort_and_filter_results {
            if let Some(keyword) = workflow.keyword.clone() {
                // TODO Don't clone the items
                workflow.response.items = filter_and_sort_items(workflow.response.items, keyword)
            }
        }

        match workflow.response.write(writer) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error writing response: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub async fn execute_async<R: AsyncRunnable>(
        config: WorkflowConfig,
        runnable: R,
        writer: &mut dyn std::io::Write,
    ) {
        let mut workflow = Workflow::new(config);
        match runnable.run_async(&mut workflow).await {
            Ok(_) => {}
            Err(e) => {
                workflow.prepend_item(e.error_item());
            }
        }

        if workflow.sort_and_filter_results {
            if let Some(keyword) = workflow.keyword.clone() {
                // TODO Don't clone the items
                workflow.response.items = filter_and_sort_items(workflow.response.items, keyword)
            }
        }

        match workflow.response.write(writer) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error writing response: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn set_filter_keyword(&mut self, keyword: String) {
        self.keyword = Some(keyword);
        self.sort_and_filter_results = true;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_run_success() {}

    #[test]
    fn test_sync_run_error() {}
}
