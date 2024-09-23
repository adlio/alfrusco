use alfrusco::{config, DefaultWorkflowError, Item, Workflow};
use clap::Parser;

#[derive(Parser)]
struct StaticOutputWorkflow {}

pub fn main() {
    env_logger::init();
    let command = StaticOutputWorkflow {};
    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl alfrusco::Runnable for StaticOutputWorkflow {
    type Error = DefaultWorkflowError;
    fn run(self, wf: &mut Workflow) -> Result<(), DefaultWorkflowError> {
        wf.response.skip_knowledge(true);
        wf.response.append_items(vec![
            Item::new("First Option").subtitle("First Subtitle"),
            Item::new("Option 2").subtitle("Second Subtitle"),
            Item::new("Three").subtitle("3"),
        ]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_output_workflow() {
        let command = StaticOutputWorkflow {};
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().into_path();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"title\":\"First Option\""));
        assert!(output.contains("\"subtitle\":\"First Subtitle\""));
    }
}
