use std::str::Utf8Error;
use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};

use self::ByteVecError::*;
use self::BVExpectedSize::*;

#[derive(Debug, Clone, Copy)]
pub enum BVExpectedSize {
    LessOrEqualThan(usize),
    MoreThan(usize),
    EqualTo(usize),
}

#[derive(Debug, Clone)]
pub enum ByteVecError {
    StringDecodeUtf8Error(Utf8Error),
    BadSizeDecodeError {
        expected: BVExpectedSize,
        actual: usize,
    },
    OverflowError,
}

impl Display for ByteVecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StringDecodeUtf8Error(utf8_error) => write!(f, "StringDecodeUtf8Error: {}", utf8_error),
            BadSizeDecodeError { expected, actual } => {
                write!(f,
                       "The size expected for the structure is {}, but the size of the given \
                        buffer is {}",
                       match expected {
                           LessOrEqualThan(expected) => format!("less or equal than {}", expected),
                           MoreThan(expected) => format!("more than {}", expected),
                           EqualTo(expected) => expected.to_string(),
                       },
                       actual)
            }
            OverflowError => {
                write!(f,
                       "OverflowError: The size of the data structure surpasses the \
                       max value of the integral generic type")
            }
        }
    }
}

impl Error for ByteVecError {

    fn cause(&self) -> Option<&dyn Error> {
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
