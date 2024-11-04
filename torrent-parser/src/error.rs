use std::{num::ParseIntError, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TorrentParserError {
    #[error("Invalid Structure:{0}")]
    InvalidStructure(String),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Missing Required Field: {0}")]
    MissingRequiredField(String),

    #[error("Invalid Field Type: expected {expected}, found {found}")]
    FieldTypeError { expected: String, found: String },

    #[error("Unknown Specifier: {0}")]
    UnknownSpecifier(u8),

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("Cannot Read File: {0}")]
    CannotReadFile(#[from] std::io::Error),
}
