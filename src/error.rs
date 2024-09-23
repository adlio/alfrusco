use std::any::type_name_of_val;

use crate::Item;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    FromUtf8(std::string::FromUtf8Error),
    ParseInt(std::num::ParseIntError),
    Serde(serde_json::Error),
    Var(std::env::VarError),
    MissingEnvVar(String),
    Workflow(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::Fmt(ref err) => write!(f, "Fmt Error: {}", err),
            Error::FromUtf8(ref err) => write!(f, "FromUtf8 Error: {}", err),
            Error::ParseInt(ref err) => write!(f, "ParseIntError: {}", err),
            Error::Serde(ref err) => write!(f, "Serde Error: {}", err),
            Error::Var(ref err) => write!(f, "Var Error: {}", err),
            Error::MissingEnvVar(ref var) => write!(f, "Missing environment variable: {}", var),
            Error::Workflow(ref msg) => write!(f, "Workflow Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Error {
        Error::Fmt(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::FromUtf8(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Error {
        Error::Var(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Fmt(ref err) => Some(err),
            Error::FromUtf8(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::Serde(ref err) => Some(err),
            Error::Var(ref err) => Some(err),
            Error::MissingEnvVar(_) => None,
            Error::Workflow(_) => None,
        }
    }
}

pub trait WorkflowError: std::error::Error + std::fmt::Display {
    fn error_item(&self) -> Item {
        match self.source() {
            Some(source) => {
                let type_name = type_name_of_val(source);
                Item::new(format!("Error: {}", self)).subtitle(type_name.to_string())
            }
            None => Item::new(format!("An error occurred: {}", self)),
        }
    }
}

impl WorkflowError for Error {
    // Default implementation is sufficient
}
