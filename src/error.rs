use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    FromUtf8(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    Serde(serde_json::Error),
    Var(std::env::VarError),
    MissingEnvVar(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::Fmt(ref err) => write!(f, "Fmt Error: {}", err),
            Error::FromUtf8(ref err) => write!(f, "FromUtf8 Error: {}", err),
            Error::ParseIntError(ref err) => write!(f, "ParseIntError: {}", err),
            Error::Serde(ref err) => write!(f, "Serde Error: {}", err),
            Error::Var(ref err) => write!(f, "Var Error: {}", err),
            Error::MissingEnvVar(ref var) => write!(f, "Missing environment variable: {}", var),
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
        Error::ParseIntError(err)
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
            Error::ParseIntError(ref err) => Some(err),
            Error::Serde(ref err) => Some(err),
            Error::Var(ref err) => Some(err),
            Error::MissingEnvVar(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct WorkflowError {
    message: String,
}

impl WorkflowError {
    pub fn new<T: Into<String>>(message: T) -> Self {
        WorkflowError {
            message: message.into(),
        }
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WorkflowError {}

// Implement From for common error types
impl From<std::io::Error> for WorkflowError {
    fn from(err: std::io::Error) -> Self {
        WorkflowError::new(err.to_string())
    }
}

impl From<String> for WorkflowError {
    fn from(err: String) -> Self {
        WorkflowError::new(err)
    }
}

impl From<&str> for WorkflowError {
    fn from(err: &str) -> Self {
        WorkflowError::new(err)
    }
}

impl From<std::fmt::Error> for WorkflowError {
    fn from(err: std::fmt::Error) -> WorkflowError {
        WorkflowError::new(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for WorkflowError {
    fn from(err: std::string::FromUtf8Error) -> WorkflowError {
        WorkflowError::new(err.to_string())
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> WorkflowError {
        WorkflowError::new(err.to_string())
    }
}

impl From<std::num::ParseIntError> for WorkflowError {
    fn from(err: std::num::ParseIntError) -> WorkflowError {
        WorkflowError::new(err.to_string())
    }
}

impl From<std::env::VarError> for WorkflowError {
    fn from(err: std::env::VarError) -> WorkflowError {
        WorkflowError::new(err.to_string())
    }
}
