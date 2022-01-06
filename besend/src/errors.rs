//! Crate-wide error implementation to standardise all potential errors

use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    MessageEnded,
    UnknownMessage(u8),
    StringLimit((u16, u16)),
    InvalidString,
    StringTooLong,
    NotListening,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MessageEnded => write!(f, "Message ended prematurely"),
            Self::UnknownMessage(unknown) => {
                write!(f, "Unknown message with #{} designator", unknown)
            }
            Self::StringLimit((limit, len)) => write!(
                f,
                "String sent over network limited to {} but was {} in length",
                limit, len
            ),
            Self::InvalidString => write!(f, "Invalid UTF-8 string sent over network"),
            Self::StringTooLong => {
                write!(f, "Couldn't encode string, it's length is >{}", u16::MAX)
            }
            Self::Io(err) => write!(f, "{}", err),
            Self::NotListening => write!(f, "Nothing selected to listen for"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
