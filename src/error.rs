///! Contains error related stuff.

use std::error;
use std::fmt;
use std::io;
use fbx_binary_reader;


pub type Result<T> = ::std::result::Result<T, Error>;

/// A type for critical errors.
///
/// "Critical" means "impossible to continue loading".
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError(fbx_binary_reader::Error),
    UnclassifiedCritical(String),
    UnsupportedVersion(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref err) => write!(f, "I/O error: {}", err),
            Error::ParseError(ref err) => write!(f, "Parse error: {}", err),
            Error::UnclassifiedCritical(ref err) => write!(f, "Unclassified critical error: {}", err),
            Error::UnsupportedVersion(ref err) => write!(f, "Unsupported version: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref err) => err.description(),
            Error::ParseError(ref err) => err.description(),
            Error::UnclassifiedCritical(_) => "Unclassified critical error",
            Error::UnsupportedVersion(_) => "Unsupported version",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err as &error::Error),
            Error::ParseError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<fbx_binary_reader::Error> for Error {
    fn from(err: fbx_binary_reader::Error) -> Error {
        Error::ParseError(err)
    }
}
