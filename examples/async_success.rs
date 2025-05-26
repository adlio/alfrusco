use alfrusco::{config, AsyncRunnable, Item, Workflow};
use clap::Parser;

#[derive(Parser, Debug)]
struct AsyncSuccessWorkflow {
    #[arg(short, long)]
    pub message: Option<String>,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let command = AsyncSuccessWorkflow::parse();
    alfrusco::execute_async(&config::AlfredEnvProvider, command, &mut std::io::stdout()).await;
}

#[async_trait::async_trait]
impl AsyncRunnable for AsyncSuccessWorkflow {
    type Error = alfrusco::Error;

    async fn run_async(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        // Small delay to simulate async work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let message = self.message.unwrap_or_else(|| "Hello from async!".to_string());
        wf.append_item(Item::new(message).subtitle("Async success workflow example"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_success_workflow() {
        let command = AsyncSuccessWorkflow { 
            message: Some("Async test message".to_string()) 
        };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Async test message"));
    }
}
