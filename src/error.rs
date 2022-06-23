use serde_json::Error as JsonError;
use std::{fmt::Display, path::Path};
use thiserror::Error as ThisError;

#[derive(Debug, Display)]
pub enum Code {
    InvalidRequest,
    CallError,
    IOError,
    UnknownError,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("gpr ({code}) {name}: {message}")]
    Gpr {
        code: Code,
        name: String,
        message: String,
    },
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    Cstring {
        #[from]
        source: std::ffi::IntoStringError,
    },
    #[error(transparent)]
    Json {
        #[from]
        source: JsonError,
    },
    #[error("invalid attribute {name} from {file}: {value}")]
    InvalidAttribute {
        file: String,
        name: String,
        value: String,
    },
    #[error("invalid attribute value {value} for attribute {name} in {file}")]
    InvalidAttributeValue {
        file: String,
        name: String,
        value: String,
    },
}

impl Error {
    pub fn from_code(code: Code, name: &str, message: &str) -> Error {
        Error::Gpr {
            code,
            name: String::from(name),
            message: String::from(message),
        }
    }

    pub fn from_status(status: i32, name: &str, message: &str) -> Option<Error> {
        match status {
            0 => None,
            1 => Some(Error::from_code(Code::InvalidRequest, name, message)),
            2 => Some(Error::from_code(Code::CallError, name, message)),
            3 => Some(Error::from_code(Code::UnknownError, name, message)),
            _ => Some(Error::from_code(
                Code::UnknownError,
                "UnknownError",
                "unknown code",
            )),
        }
    }

    pub fn invalid_attribute(file: &Path, attribute: &str, value: &str) -> Error {
        Error::InvalidAttribute {
            file: String::from(file.to_str().unwrap()),
            name: String::from(attribute),
            value: String::from(value),
        }
    }

    pub fn invalid_attribute_value(
        file: &Path,
        attribute: &str,
        value: &crate::binding::AttributeValue,
    ) -> Error {
        Error::InvalidAttributeValue {
            file: String::from(file.to_str().unwrap()),
            name: String::from(attribute),
            value: format!("{value}"),
        }
    }
}
