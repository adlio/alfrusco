use std::time::Duration;

use alfrusco::{DefaultWorkflowError, URLItem, Workflow, WorkflowConfig, WorkflowError};

struct URLItemsWorkflow {}
pub fn main() {
    env_logger::init();
    let config = WorkflowConfig::from_env().unwrap();
    let workflow = URLItemsWorkflow {};
    Workflow::execute(config, workflow, &mut std::io::stdout());
}

impl alfrusco::Runnable for URLItemsWorkflow {
    type Error = DefaultWorkflowError;
    fn run(self, wf: &mut Workflow) -> Result<(), DefaultWorkflowError> {
        wf.response.skip_knowledge(true);
        wf.response.cache(Duration::from_secs(60), true);
        wf.response.append_items(vec![
            URLItem::new("DuckDuckGo", "https://www.duckduckgo.com").into(),
            URLItem::new("Google", "https://www.google.com").into(),
        ]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_items_workflow() {
        let config = WorkflowConfig::for_testing().unwrap();
        let workflow = URLItemsWorkflow {};
        let mut buffer = Vec::new();
        alfrusco::Workflow::execute(config, workflow, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"title\":\"DuckDuckGo\""));
        assert!(output.contains("\"cache\":{\"seconds\":60.0,\"loosereload\":true}"));
    }
}
