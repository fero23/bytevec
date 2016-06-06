use std::str::Utf8Error;
use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};

use self::ByteVecError::*;
use self::BVWantedSize::*;

#[derive(Debug, Clone, Copy)]
pub enum BVWantedSize {
    MoreThan(usize),
    EqualTo(usize),
}

#[derive(Debug, Clone)]
pub enum ByteVecError {
    StringDecodeUtf8Error(Utf8Error),
    BadSizeDecodeError {
        wanted: BVWantedSize,
        actual: usize,
    },
    OverflowError,
}

impl Display for ByteVecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StringDecodeUtf8Error(utf8_error) => write!(f, "StringDecodeUtf8Error: {}", utf8_error),
            BadSizeDecodeError { wanted, actual } => {
                write!(f,
                       "The size specified for the structure is {}, but the size of the given \
                        buffer is {}",
                       match wanted {
                           MoreThan(wanted) => format!("more than {}", wanted),
                           EqualTo(wanted) => wanted.to_string(),
                       },
                       actual)
            }
            OverflowError => {
                write!(f,
                       "OverflowError: The size of the data structure surpasses the u32 max size \
                        (4GB)")
            }
        }
    }
}

impl Error for ByteVecError {
    fn description(&self) -> &str {
        match *self {
            StringDecodeUtf8Error(ref utf8_error) => utf8_error.description(),
            BadSizeDecodeError { .. } => {
                "the size specified for the structure differs from the size of the given buffer"
            }
            OverflowError => "the size of the data structure surpasses the u32 max size",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            StringDecodeUtf8Error(ref utf8_error) => Some(utf8_error),
            _ => None,
        }
    }
}

impl From<Utf8Error> for ByteVecError {
    fn from(error: Utf8Error) -> ByteVecError {
        StringDecodeUtf8Error(error)
    }
}
