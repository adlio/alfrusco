use std::time::Duration;

use alfrusco::{config, DefaultWorkflowError, URLItem, Workflow};
use clap::Parser;

#[derive(Parser)]
struct URLItemsWorkflow {}

pub fn main() {
    env_logger::init();
    let command = URLItemsWorkflow {};
    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
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
        let command = URLItemsWorkflow {};
        let dir = tempfile::tempdir().unwrap().into_path();
        let mut buffer = Vec::new();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        println!("URL items: {}", output);
        assert!(output.contains("\"title\":\"DuckDuckGo\""));
        assert!(output.contains("\"cache\":{\"seconds\":60,\"loosereload\":true}"));
    }
}
