use std::time::Duration;

use alfrusco::{config, AsyncRunnable, Item, Workflow, WorkflowError};
use clap::Parser;

#[derive(Parser, Debug)]
struct AsyncErrorWorkflow {
    #[arg(short, long)]
    pub url: Option<String>,

    #[arg(short, long)]
    pub timeout: Option<u64>,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let command = AsyncErrorWorkflow::parse();
    alfrusco::execute_async(&config::AlfredEnvProvider, command, &mut std::io::stdout()).await;
}

#[async_trait::async_trait]
impl AsyncRunnable for AsyncErrorWorkflow {
    type Error = AsyncErrorWorkflowError;

    async fn run_async(self, wf: &mut Workflow) -> Result<(), AsyncErrorWorkflowError> {
        // If timeout is provided, simulate a timeout error
        if let Some(timeout) = self.timeout {
            tokio::time::sleep(Duration::from_secs(timeout)).await;
            return Err(AsyncErrorWorkflowError::Timeout(format!(
                "Operation timed out after {} seconds",
                timeout
            )));
        }

        // If URL is provided, try to fetch it (likely to cause errors with invalid URLs)
        if let Some(url) = self.url {
            let client = reqwest::Client::new();
            let response = client
                .get(&url)
                .timeout(Duration::from_secs(5))
                .send()
                .await
                .map_err(|e| AsyncErrorWorkflowError::Request(e))?;

            if !response.status().is_success() {
                return Err(AsyncErrorWorkflowError::StatusCode(
                    response.status().as_u16(),
                ));
            }

            let body = response
                .text()
                .await
                .map_err(|e| AsyncErrorWorkflowError::Request(e))?;
            wf.append_item(Item::new(format!("Response: {} bytes", body.len())));
            Ok(())
        } else {
            // Demonstrate a custom error
            Err(AsyncErrorWorkflowError::Custom(
                "No URL provided".to_string(),
            ))
        }
    }
}

#[derive(Debug)]
pub enum AsyncErrorWorkflowError {
    Request(reqwest::Error),
    StatusCode(u16),
    Timeout(String),
    Custom(String),
}

impl From<reqwest::Error> for AsyncErrorWorkflowError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl WorkflowError for AsyncErrorWorkflowError {}

impl std::fmt::Display for AsyncErrorWorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncErrorWorkflowError::Request(e) => write!(f, "Request error: {}", e),
            AsyncErrorWorkflowError::StatusCode(code) => {
                write!(f, "HTTP error: status code {}", code)
            }
            AsyncErrorWorkflowError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            AsyncErrorWorkflowError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AsyncErrorWorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AsyncErrorWorkflowError::Request(e) => Some(e),
            AsyncErrorWorkflowError::StatusCode(_) => None,
            AsyncErrorWorkflowError::Timeout(_) => None,
            AsyncErrorWorkflowError::Custom(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_error_workflow_no_url() {
        let command = AsyncErrorWorkflow {
            url: None,
            timeout: None,
        };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Error: No URL provided"));
    }

    #[tokio::test]
    async fn test_async_error_workflow_invalid_url() {
        let command = AsyncErrorWorkflow {
            url: Some("invalid://url".to_string()),
            timeout: None,
        };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Request error:"));
    }
}
