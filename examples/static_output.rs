use alfrusco::{Item, Workflow, WorkflowConfig, WorkflowResult};

struct StaticOutputWorkflow {}

pub fn main() {
    let config = WorkflowConfig::from_env().unwrap();
    let workflow = StaticOutputWorkflow {};
    Workflow::execute(config, workflow, &mut std::io::stdout());
}

impl alfrusco::Runnable for StaticOutputWorkflow {
    fn run(self, wf: &mut Workflow) -> WorkflowResult {
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
        let config = WorkflowConfig::for_testing().unwrap();
        let workflow = StaticOutputWorkflow {};
        let mut buffer = Vec::new();
        alfrusco::Workflow::execute(config, workflow, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"title\":\"First Option\""));
        assert!(output.contains("\"subtitle\":\"First Subtitle\""));
    }
}
