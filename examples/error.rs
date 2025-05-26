use alfrusco::{config, Item, Workflow, WorkflowError};
use clap::Parser;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
struct ErrorWorkflow {
    #[arg(short, long)]
    pub file_path: Option<String>,
}

pub fn main() {
    env_logger::init();
    let command = ErrorWorkflow::parse();
    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl alfrusco::Runnable for ErrorWorkflow {
    type Error = ErrorWorkflowError;
    
    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        // This will deliberately cause an error if file_path is provided
        if let Some(file_path) = self.file_path {
            // Try to open a file that likely doesn't exist
            let mut file = File::open(&file_path).map_err(|e| ErrorWorkflowError::Io(e))?;
            
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| ErrorWorkflowError::Io(e))?;
            
            wf.append_item(Item::new(format!("File content: {}", content)));
            Ok(())
        } else {
            // Demonstrate a custom error
            Err(ErrorWorkflowError::Custom("No file path provided".to_string()))
        }
    }
}

#[derive(Debug)]
pub enum ErrorWorkflowError {
    Io(std::io::Error),
    Custom(String),
}

impl From<std::io::Error> for ErrorWorkflowError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl WorkflowError for ErrorWorkflowError {}

impl std::fmt::Display for ErrorWorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorWorkflowError::Io(e) => write!(f, "IO error: {}", e),
            ErrorWorkflowError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ErrorWorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorWorkflowError::Io(e) => Some(e),
            ErrorWorkflowError::Custom(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_workflow_no_file() {
        let command = ErrorWorkflow { file_path: None };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Error: No file path provided"));
    }

    #[test]
    fn test_error_workflow_nonexistent_file() {
        let command = ErrorWorkflow { 
            file_path: Some("nonexistent_file.txt".to_string()) 
        };
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("IO error:"));
    }
}
