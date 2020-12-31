use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Syntax,
    UnsupportedType,
    TrailingCharacters,
    Eof
}
// impl std::Error for Error {}
impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Syntax => formatter.write_str("unexpected syntax error"),
            Error::UnsupportedType => formatter.write_str("unsupported data type"),
            Error::TrailingCharacters => formatter.write_str("EOF expected, found extra characters"),
            Error::Eof => formatter.write_str("Expected extra characters, found EOF")
        }
    }
}
impl std::error::Error for Error {}