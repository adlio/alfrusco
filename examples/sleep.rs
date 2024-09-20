use std::{process::Command, time::Duration};

use clap::Parser;

use alfrusco::{DefaultWorkflowError, URLItem, Workflow, WorkflowConfig};

#[derive(Parser, Debug)]
struct SleepCommand {
    #[arg(short, long)]
    duration_in_seconds: u64,
}

pub fn main() {
    let command = SleepCommand::parse();
    let config = WorkflowConfig::from_env().unwrap();
    Workflow::execute(config, command, &mut std::io::stdout());
}

impl alfrusco::Runnable for SleepCommand {
    type Error = DefaultWorkflowError;
    fn run(self, wf: &mut Workflow) -> Result<(), DefaultWorkflowError> {
        wf.response.skip_knowledge(true);
        wf.response.rerun(Duration::from_millis(500));

        let mut cmd = Command::new("/bin/sleep");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.arg("5");

        wf.run_in_background("sleep", Duration::from_secs(self.duration_in_seconds), cmd);

        wf.response
            .append_items(vec![URLItem::new("Google", "https://www.google.com").into()]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_workflow() {
        let config = WorkflowConfig::for_testing().unwrap();
        let workflow = SleepCommand {
            duration_in_seconds: 5,
        };
        let mut buffer = Vec::new();
        alfrusco::Workflow::execute(config, workflow, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"title\":\"Google\""));
    }
}
