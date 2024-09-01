use std::path::PathBuf;

use crate::Result;
use crate::WorkflowConfig;
use crate::{handle, Item, Response};

pub struct Workflow {
    pub config: WorkflowConfig,
    pub response: Response,
    pub writer: Box<dyn std::io::Write>,
}

impl Workflow {
    /// Creates a new Workflow instance by reading the environment variables.
    /// This will fail with an Error::VarError if any of the required
    /// environment variables are not set.
    ///
    pub fn new(config: WorkflowConfig) -> Workflow {
        Workflow {
            config,
            response: Response::new(),
            writer: Box::new(std::io::stdout()),
        }
    }

    pub fn new_from_env() -> Result<Workflow> {
        let config = WorkflowConfig::from_env()?;
        Ok(Workflow::new(config))
    }

    pub fn new_for_testing() -> Result<Workflow> {
        let config = WorkflowConfig::for_testing()?;
        Ok(Workflow::new(config))
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.config.workflow_cache
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.config.workflow_data
    }

    /// run accepts a function which takes a mutable borrow of the
    /// Workflow, and returns a Result. The function is expected to
    /// call methods on the Workflow or its response. If the function
    /// returns an error, that error is prepended as an error item
    /// in the response.
    pub fn run(
        config: WorkflowConfig,
        f: impl FnOnce(&mut Workflow) -> std::result::Result<(), Box<dyn std::error::Error>>,
    ) {
        let mut workflow = Workflow::new(config);

        // If the response includes alfrusco clipboard instructions, handle them
        // first
        handle();

        let result = f(&mut workflow);
        if let Err(err) = result {
            let error_item = Item::new(format!("Error: {}", err))
                .subtitle("Check the logs for more information.");
            workflow.response.prepend_items(vec![error_item]);
        }
        match workflow.response.write(&mut workflow.writer) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error writing response: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_workflow() {
        let _wf = Workflow::new_for_testing();
    }
}
