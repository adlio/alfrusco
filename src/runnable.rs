use async_trait::async_trait;

use crate::config::ConfigProvider;
use crate::workflow::{finalize_workflow, setup_workflow};
use crate::{Workflow, WorkflowError};

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
