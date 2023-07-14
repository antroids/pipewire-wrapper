use std::fmt::{Debug, Display, Formatter, Pointer};

use crate::spa::pod::PodError;

pub enum Error {
    ErrorCode(u32),
    ErrorMessage(&'static str),
    MethodCallOnNull(),
    MethodNotFound(String),
    VersionMismatch(u32, u32),
    TypeMismatch,
    WrongTimeFormat,
    CannotCreateInstance,
    NullPointer,
    PodParseError(PodError),
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ErrorCode(code) => write!(f, "ErrorCode({})", code),
            Error::ErrorMessage(message) => write!(f, "ErrorMessage({})", message),
            Error::MethodCallOnNull() => write!(f, "MethodCallOnNull"),
            Error::MethodNotFound(method) => {
                write!(f, "MethodNotFound: method {} not found", method)
            }
            Error::VersionMismatch(expected, actual) => write!(
                f,
                "Version too low, expected {}, actual {}",
                expected, actual
            ),
            Error::TypeMismatch => write!(f, "TypeMismatch"),
            Error::WrongTimeFormat => write!(f, "WrongTimeFormat"),
            Error::CannotCreateInstance => write!(f, "CannotCreateInstance"),
            Error::NullPointer => write!(f, "NullPointer"),
            Error::PodParseError(pod_error) => write!(f, "PodParseError({:?})", pod_error),
        }
    }
}

impl std::error::Error for Error {}
