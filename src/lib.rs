// External crate dependencies
use async_trait::async_trait;

// Internal modules
mod background;
mod background_job;
mod clipboard;
mod error;
mod icon;
mod item;
mod response;
mod url_item;
mod workflow;

// Pub re-exports
pub mod config;
pub use self::{
    error::{DefaultWorkflowError, Error, Result, WorkflowError},
    icon::{Icon, *},
    item::{Arg, Item, Key, Modifier},
    response::Response,
    url_item::URLItem,
    workflow::Workflow,
};

use item::filter_and_sort_items;

pub fn handle() {
    clipboard::handle_clipboard()
}

use crate::clipboard::handle_clipboard;
use crate::config::ConfigProvider;

pub trait Runnable {
    type Error: WorkflowError;
    fn run(self, workflow: &mut Workflow) -> std::result::Result<(), Self::Error>;
}

#[async_trait]
pub trait AsyncRunnable {
    type Error: WorkflowError;
    async fn run_async(self, workflow: &mut Workflow) -> std::result::Result<(), Self::Error>;
}

pub fn execute<R: Runnable>(
    provider: &dyn ConfigProvider,
    runnable: R,
    writer: &mut dyn std::io::Write,
) {
    let mut workflow = setup_workflow(provider);
    if let Err(e) = runnable.run(&mut workflow) {
        workflow.prepend_item(e.error_item());
    }
    finalize_workflow(workflow, writer);
}

pub async fn execute_async<R: AsyncRunnable>(
    provider: &dyn ConfigProvider,
    runnable: R,
    writer: &mut dyn std::io::Write,
) {
    let mut workflow = setup_workflow(provider);
    if let Err(e) = runnable.run_async(&mut workflow).await {
        workflow.prepend_item(e.error_item());
    }
    finalize_workflow(workflow, writer);
}

fn setup_workflow(provider: &dyn ConfigProvider) -> Workflow {
    handle_clipboard();
    let config = provider.config();
    if config.is_err() {
        eprintln!("Error loading config: {}", config.unwrap_err());
        std::process::exit(1);
    }
    match Workflow::new(config.unwrap()) {
        Ok(workflow) => workflow,
        Err(e) => {
            eprintln!("Error creating workflow: {}", e);
            std::process::exit(1);
        }
    }
}

fn finalize_workflow(mut workflow: Workflow, writer: &mut dyn std::io::Write) {
    if workflow.sort_and_filter_results {
        if let Some(keyword) = workflow.keyword.clone() {
            workflow.response.items = filter_and_sort_items(workflow.response.items, keyword);
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
