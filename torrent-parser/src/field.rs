use std::{collections::HashMap, iter::Peekable, vec::IntoIter};

use sha1::{Digest, Sha1};

use crate::error::TorrentParserError;

pub(crate) enum Field {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<Field>),
    Dict(HashMap<String, Field>),
}

impl Field {
    pub fn field_type(&self) -> String {
        match self {
            Field::String(_) => "String".to_string(),
            Field::Integer(_) => "Integer".to_string(),
            Field::List(_) => "List".to_string(),
            Field::Dict(_) => "Dict".to_string(),
        }
    }
}

pub(crate) fn get_field_type(
    buffer: &mut Peekable<IntoIter<u8>>,
) -> Result<Option<Field>, TorrentParserError> {
    let specifier = buffer.next();

    match specifier {
        None => Ok(None),
        Some(c) => {
            if c.is_ascii_digit() {
                // get until the colon
                let mut length = Vec::new();
                length.push(c);
                loop {
                    match buffer.next() {
                        Some(c) => {
                            if c.is_ascii_digit() {
                                length.push(c);
                            } else if c == b':' {
                                break;
                            } else {
                                return Err(TorrentParserError::InvalidStructure(
                                    "Expected colon for string".to_string(),
                                ));
                            }
                        }
                        None => {
                            return Err(TorrentParserError::InvalidStructure(
                                "Unexpected end for string length".to_string(),
                            ))
                        }
                    }
                }
                let length = String::from_utf8(length)?;
                let length = length.parse::<usize>().unwrap();
                let mut field = Vec::new();
                for i in 0..length {
                    match buffer.next() {
                        Some(c) => field.push(c),
                        None => {
                            return Err(TorrentParserError::InvalidStructure(format!(
                                "Unexpected end for string, expected length {}, ending at {}",
                                length, i
                            )))
                        }
                    }
                }
                Ok(Some(Field::String(field)))
            } else if c == b'i' {
                // get until the e
                let mut field = Vec::new();
                loop {
                    match buffer.next() {
                        Some(c) => {
                            if c == b'e' {
                                break;
                            } else {
                                field.push(c);
                            }
                        }
                        None => {
                            return Err(TorrentParserError::InvalidStructure(
                                "Unexpected end for integer".to_string(),
                            ))
                        }
                    }
                }
                let field = String::from_utf8(field)?;
                let field = field.parse::<i64>()?;
                Ok(Some(Field::Integer(field)))
            } else if c == b'l' {
                // list
                let mut list = Vec::new();
                loop {
                    let peek_next = buffer.peek().ok_or(TorrentParserError::InvalidStructure(
                        "Unexpected end for list".to_string(),
                    ))?;
                    if *peek_next == b'e' {
                        buffer.next();
                        break;
                    }
                    match get_field_type(buffer)? {
                        Some(field) => list.push(field),
                        None => {
                            return Err(TorrentParserError::InvalidStructure(
                                "Unexpected end for list".to_string(),
                            ))
                        }
                    }
                }
                Ok(Some(Field::List(list)))
            } else if c == b'd' {
                // dictionary
                let mut dict = HashMap::new();
                loop {
                    let peek_next = buffer.peek().ok_or(TorrentParserError::InvalidStructure(
                        "Unexpected end for dict".to_string(),
                    ))?;
                    if *peek_next == b'e' {
                        buffer.next();
                        break;
                    }
                    match get_field_type(buffer)? {
                        Some(field) => {
                            let key = match field {
                                Field::String(key) => String::from_utf8(key)?,
                                other => {
                                    return Err(TorrentParserError::FieldTypeError {
                                        expected: "String".to_string(),
                                        found: other.field_type(),
                                    });
                                }
                            };
                            match get_field_type(buffer)? {
                                Some(value) => {
                                    dict.insert(key, value);
                                }
                                None => {
                                    return Err(TorrentParserError::InvalidStructure(
                                        "Expected value for dictionary".to_string(),
                                    ))
                                }
                            }
                        }
                        None => break,
                    }
                }
                Ok(Some(Field::Dict(dict)))
            } else {
                Err(TorrentParserError::UnknownSpecifier(c))
            }
        }
    }
}

pub(crate) fn extract_info_hash(bencoded: &[u8]) -> Result<Vec<u8>, TorrentParserError> {
    let mut buffer = bencoded.iter().peekable();

    loop {
        match buffer.next() {
            Some(c) => {
                if *c == b'4' {
                    let mut slice = Vec::new();
                    slice.push(*c);
                    for _ in 0..=4 {
                        match buffer.next() {
                            Some(c) => slice.push(*c),
                            None => {
                                return Err(TorrentParserError::InvalidStructure(
                                    "Unexpected end for info".to_string(),
                                ))
                            }
                        }
                    }
                    if slice == b"4:info" {
                        break;
                    }
                }
            }
            None => {
                return Err(TorrentParserError::InvalidStructure(
                    "Unexpected end for info".to_string(),
                ))
            }
        }
    }
    let mut info_text = Vec::new();
    loop {
        match buffer.next() {
            Some(c) => {
                if *c == b'e' {
                    break;
                } else {
                    info_text.push(*c);
                }
            }
            None => {
                return Err(TorrentParserError::InvalidStructure(
                    "Unexpected end for info".to_string(),
                ))
            }
        }
    }
    let mut hasher = Sha1::new();
    hasher.update(info_text);
    let result = hasher.finalize();
    Ok(result.to_vec())
}
