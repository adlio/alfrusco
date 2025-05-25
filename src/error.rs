use std::any::type_name_of_val;

use thiserror::Error;

use crate::Item;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Fmt Error: {0}")]
    Fmt(#[from] std::fmt::Error),
    
    #[error("FromUtf8 Error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    
    #[error("ParseIntError: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Var Error: {0}")]
    Var(#[from] std::env::VarError),
    
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Workflow Error: {0}")]
    Workflow(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Implement From<String> and From<&str> manually since they're not automatically derived
impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::Workflow(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::Workflow(msg.to_string())
    }
}

pub trait WorkflowError: std::error::Error + std::fmt::Display {
    fn error_item(&self) -> Item {
        match self.source() {
            Some(source) => {
                let type_name = type_name_of_val(source);
                Item::new(format!("Error: {self}")).subtitle(type_name.to_string())
            }
            None => Item::new(format!("An error occurred: {self}")),
        }
    }
}

impl WorkflowError for Error {
    // Default implementation is sufficient
}
