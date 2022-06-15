use std::fmt;
use std::fmt::Display;

#[derive(Debug, Display)]
pub enum Code {
    InvalidRequest,
    CallError,
    IOError,
    UnknownError,
}

#[derive(Debug)]
pub struct Error {
    code: Code,
    name: String,
    message: String,
}

impl Error {
    pub fn new(code: Code, name: &str, message: &str) -> Error {
        Error {
            code: code,
            name: String::from(name),
            message: String::from(message),
        }
    }

    pub fn from_status(status: i32, name: &str, message: &str) -> Option<Error> {
        match status {
            0 => None,
            1 => Some(Error::new(Code::InvalidRequest, &name, &message)),
            2 => Some(Error::new(Code::CallError, &name, &message)),
            3 => Some(Error::new(Code::UnknownError, &name, &message)),
            _ => Some(Error::new(
                Code::UnknownError,
                "UnknownError",
                "unknown error",
            )),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(Code::IOError, "IOError", &error.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (code {}): {}", self.name, self.code, self.message)
    }
}
