#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    FromUtf8(std::string::FromUtf8Error),
    Serde(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::Fmt(ref err) => write!(f, "Fmt Error: {}", err),
            Error::FromUtf8(ref err) => write!(f, "FromUtf8 Error: {}", err),
            Error::Serde(ref err) => write!(f, "Serde Error: {}", err),
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Fmt(ref err) => Some(err),
            Error::FromUtf8(ref err) => Some(err),
            Error::Serde(ref err) => Some(err),
        }
    }
}
