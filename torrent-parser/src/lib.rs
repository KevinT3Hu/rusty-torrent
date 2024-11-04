use error::TorrentParserError;
use field::{extract_info_hash, get_field_type, Field};
use model::{Info, TorrentMetadata, TrackerResponse, TrackerResponsePeer, TrackerSuccessResponse};

pub mod error;
mod field;
pub mod model;

pub fn parse_torrent_metadata(bencoded: Vec<u8>) -> Result<TorrentMetadata, TorrentParserError> {
    let info_hash = extract_info_hash(&bencoded)?;

    let mut iter = bencoded.into_iter().peekable();
    let parsed_structure = get_field_type(&mut iter)?.ok_or(
        TorrentParserError::InvalidStructure("Expected field".to_string()),
    )?;

    // the root element should be a dictionary
    let dict = match parsed_structure {
        Field::Dict(dict) => dict,
        other => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Dict".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read announce
    let announce = match dict.get("announce") {
        Some(Field::String(announce)) => String::from_utf8(announce.clone())?,
        _ => {
            return Err(TorrentParserError::MissingRequiredField(
                "announce".to_string(),
            ))
        }
    };

    // read optional announce-list
    let announce_list = match dict.get("announce-list") {
        Some(Field::List(announce_list)) => {
            let mut announce_list = announce_list
                .iter()
                .map(|field| match field {
                    Field::List(list) => list
                        .iter()
                        .map(|field| match field {
                            Field::String(announce) => Ok(String::from_utf8(announce.clone())?),
                            other => Err(TorrentParserError::FieldTypeError {
                                expected: "String".to_string(),
                                found: other.field_type(),
                            }),
                        })
                        .collect::<Result<Vec<String>, TorrentParserError>>(),
                    _ => Err(TorrentParserError::FieldTypeError {
                        expected: "List".to_string(),
                        found: field.field_type(),
                    }),
                })
                .collect::<Result<Vec<Vec<String>>, TorrentParserError>>()?;
            announce_list.sort();
            announce_list.dedup();
            Some(announce_list)
        }
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "List".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional comment
    let comment = match dict.get("comment") {
        Some(Field::String(comment)) => Some(String::from_utf8(comment.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional created by
    let created_by = match dict.get("created by") {
        Some(Field::String(create_by)) => Some(String::from_utf8(create_by.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional creation date
    let creation_date = match dict.get("creation date") {
        Some(Field::Integer(creation_date)) => Some(*creation_date),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional encoding
    let encoding = match dict.get("encoding") {
        Some(Field::String(encoding)) => Some(String::from_utf8(encoding.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read info
    let info = match dict.get("info") {
        Some(Field::Dict(info)) => info,
        None => return Err(TorrentParserError::MissingRequiredField("info".to_string())),
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Dict".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read piece length
    let piece_length = match info.get("piece length") {
        Some(Field::Integer(piece_length)) => *piece_length,
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "piece length".to_string(),
            ))
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read pieces
    let pieces = match info.get("pieces") {
        Some(Field::String(pieces)) => pieces,
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "pieces".to_string(),
            ))
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    // divide pieces into 20-byte SHA1 hashes
    let pieces = pieces
        .chunks(20)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<u8>>>();

    // read optional private
    let private = match info.get("private") {
        Some(Field::Integer(private)) => Some(*private != 0),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read name
    let name = match info.get("name") {
        Some(Field::String(name)) => String::from_utf8(name.clone())?,
        None => return Err(TorrentParserError::MissingRequiredField("name".to_string())),
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional files
    let files = match info.get("files") {
        Some(Field::List(files)) => {
            let files = files
                .iter()
                .map(|file| match file {
                    Field::Dict(file) => {
                        let length = match file.get("length") {
                            Some(Field::Integer(length)) => *length,
                            None => {
                                return Err(TorrentParserError::MissingRequiredField(
                                    "length".to_string(),
                                ))
                            }
                            Some(other) => {
                                return Err(TorrentParserError::FieldTypeError {
                                    expected: "Integer".to_string(),
                                    found: other.field_type(),
                                })
                            }
                        };

                        let md5sum = match file.get("md5sum") {
                            Some(Field::String(md5sum)) => Some(String::from_utf8(md5sum.clone())?),
                            None => None,
                            Some(other) => {
                                return Err(TorrentParserError::FieldTypeError {
                                    expected: "String".to_string(),
                                    found: other.field_type(),
                                })
                            }
                        };

                        let path = match file.get("path") {
                            Some(Field::List(path)) => path
                                .iter()
                                .map(|field| match field {
                                    Field::String(path) => Ok(String::from_utf8(path.clone())?),
                                    other => Err(TorrentParserError::FieldTypeError {
                                        expected: "String".to_string(),
                                        found: other.field_type(),
                                    }),
                                })
                                .collect::<Result<Vec<String>, TorrentParserError>>()?,
                            None => {
                                return Err(TorrentParserError::MissingRequiredField(
                                    "path".to_string(),
                                ))
                            }
                            Some(other) => {
                                return Err(TorrentParserError::FieldTypeError {
                                    expected: "List".to_string(),
                                    found: other.field_type(),
                                })
                            }
                        };

                        Ok(Some(model::InfoFile {
                            length,
                            md5sum,
                            path,
                        }))
                    }
                    _ => Err(TorrentParserError::FieldTypeError {
                        expected: "Dict".to_string(),
                        found: file.field_type(),
                    }),
                })
                .collect::<Result<Vec<Option<model::InfoFile>>, TorrentParserError>>()?;
            Some(files.into_iter().flatten().collect())
        }
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "List".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional length
    let length = match info.get("length") {
        Some(Field::Integer(length)) => Some(*length),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    // read optional md5sum
    let md5sum = match info.get("md5sum") {
        Some(Field::String(md5sum)) => Some(String::from_utf8(md5sum.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    Ok(TorrentMetadata {
        announce,
        announce_list,
        comment,
        created_by,
        creation_date,
        encoding,
        info: Info {
            piece_length,
            pieces,
            private,
            name,
            files,
            length,
            md5sum,
        },
        info_hash,
    })
}

pub fn parse_torrent_file(file_path: &str) -> Result<TorrentMetadata, TorrentParserError> {
    let bencoded = std::fs::read(file_path)?;
    parse_torrent_metadata(bencoded)
}

pub fn parse_tracker_response(bencoded: Vec<u8>) -> Result<TrackerResponse, TorrentParserError> {
    let mut bencoded = bencoded.into_iter().peekable();
    let parsed_structure = get_field_type(&mut bencoded)?.ok_or(
        TorrentParserError::InvalidStructure("Expected field".to_string()),
    )?;

    let root_field = match parsed_structure {
        Field::Dict(dict) => dict,
        other => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Dict".to_string(),
                found: other.field_type(),
            })
        }
    };

    let failure_reason = match root_field.get("failure reason") {
        Some(Field::String(failure_reason)) => Some(String::from_utf8(failure_reason.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    if let Some(msg) = failure_reason {
        return Ok(TrackerResponse::Failure(msg));
    }

    let warning_message = match root_field.get("warning message") {
        Some(Field::String(warning_message)) => Some(String::from_utf8(warning_message.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    if let Some(msg) = warning_message {
        return Ok(TrackerResponse::Warning(msg));
    }

    let interval = match root_field.get("interval") {
        Some(Field::Integer(interval)) => *interval,
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "interval".to_string(),
            ))
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    let min_interval = match root_field.get("min interval") {
        Some(Field::Integer(min_interval)) => Some(*min_interval),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    let tracker_id = match root_field.get("tracker id") {
        Some(Field::String(tracker_id)) => Some(String::from_utf8(tracker_id.clone())?),
        None => None,
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "String".to_string(),
                found: other.field_type(),
            })
        }
    };

    let complete = match root_field.get("complete") {
        Some(Field::Integer(complete)) => *complete,
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "complete".to_string(),
            ))
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    let incomplete = match root_field.get("incomplete") {
        Some(Field::Integer(incomplete)) => *incomplete,
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "incomplete".to_string(),
            ))
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "Integer".to_string(),
                found: other.field_type(),
            })
        }
    };

    let peers = match root_field.get("peers") {
        Some(Field::List(peers)) => peers
            .iter()
            .map(|peer| match peer {
                Field::Dict(peer) => {
                    let peer_id = match peer.get("peer id") {
                        Some(Field::String(peer_id)) => Some(String::from_utf8(peer_id.clone())?),
                        None => None,
                        Some(other) => {
                            return Err(TorrentParserError::FieldTypeError {
                                expected: "String".to_string(),
                                found: other.field_type(),
                            })
                        }
                    };

                    let ip = match peer.get("ip") {
                        Some(Field::String(ip)) => String::from_utf8(ip.clone())?,
                        None => {
                            return Err(TorrentParserError::MissingRequiredField("ip".to_string()))
                        }
                        Some(other) => {
                            return Err(TorrentParserError::FieldTypeError {
                                expected: "String".to_string(),
                                found: other.field_type(),
                            })
                        }
                    };

                    let port = match peer.get("port") {
                        Some(Field::Integer(port)) => *port,
                        None => {
                            return Err(TorrentParserError::MissingRequiredField(
                                "port".to_string(),
                            ))
                        }
                        Some(other) => {
                            return Err(TorrentParserError::FieldTypeError {
                                expected: "Integer".to_string(),
                                found: other.field_type(),
                            })
                        }
                    };

                    Ok(TrackerResponsePeer { peer_id, ip, port })
                }
                _ => Err(TorrentParserError::FieldTypeError {
                    expected: "Dict".to_string(),
                    found: peer.field_type(),
                }),
            })
            .collect::<Result<Vec<TrackerResponsePeer>, TorrentParserError>>()?,
        Some(Field::String(peers_str)) => {
            // verify that the string length is a multiple of 6
            if peers_str.len() % 6 != 0 {
                return Err(TorrentParserError::InvalidStructure(
                    "Invalid length for peers string".to_string(),
                ));
            }
            let peers_chunks = peers_str
                .chunks(6)
                .map(|peer_str| {
                    let ip = format!(
                        "{}.{}.{}.{}",
                        peer_str[0], peer_str[1], peer_str[2], peer_str[3]
                    );
                    let port = u16::from_be_bytes([peer_str[4], peer_str[5]]);
                    TrackerResponsePeer {
                        peer_id: None,
                        ip,
                        port: port as i64,
                    }
                })
                .collect::<Vec<TrackerResponsePeer>>();

            peers_chunks
        }
        Some(other) => {
            return Err(TorrentParserError::FieldTypeError {
                expected: "List or String".to_string(),
                found: other.field_type(),
            })
        }
        None => {
            return Err(TorrentParserError::MissingRequiredField(
                "peers".to_string(),
            ))
        }
    };

    let resp = TrackerSuccessResponse {
        interval,
        min_interval,
        tracker_id,
        complete,
        incomplete,
        peers,
    };

    Ok(TrackerResponse::Success(resp))
}
