use alfrusco::{config, Item, Workflow};
use clap::Parser;

#[derive(Parser, Debug)]
struct SuccessWorkflow {
    #[arg(short, long)]
    pub message: Option<String>,
}

pub fn main() {
    env_logger::init();
    let command = SuccessWorkflow::parse();
    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl alfrusco::Runnable for SuccessWorkflow {
    type Error = alfrusco::Error;
    
    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        let message = self.message.unwrap_or_else(|| "Hello, World!".to_string());
        wf.append_item(Item::new(message).subtitle("Success workflow example"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_workflow() {
        let command = SuccessWorkflow { 
            message: Some("Test message".to_string()) 
        };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Test message"));
    }
}
