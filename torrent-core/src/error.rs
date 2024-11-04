use thiserror::Error;
use torrent_parser::error::TorrentParserError;

use crate::session::TorrentId;

#[derive(Error, Debug)]
pub enum RustyTorrentError {
    #[error("Torrent Parser Error: {0}")]
    TorrentParserError(#[from] TorrentParserError),

    #[error("Torrent Not Found: {0}")]
    TorrentNotFound(TorrentId),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type RustyTorrentResult<T> = Result<T, RustyTorrentError>;
