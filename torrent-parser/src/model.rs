pub struct InfoFile {
    pub length: i64,
    pub md5sum: Option<String>,
    pub path: Vec<String>,
}

pub struct Info {
    pub piece_length: i64,
    pub pieces: Vec<Vec<u8>>,
    pub private: Option<bool>,
    pub name: String,
    pub files: Option<Vec<InfoFile>>,
    pub length: Option<i64>,
    pub md5sum: Option<String>,
}

pub struct TorrentMetadata {
    pub announce: String,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub creation_date: Option<i64>,
    pub encoding: Option<String>,
    pub info: Info,
    pub info_hash: Vec<u8>,
}

impl TorrentMetadata {
    pub fn is_single_file(&self) -> bool {
        self.info.files.is_none()
    }
}

pub struct TrackerResponsePeer {
    pub peer_id: Option<String>,
    pub ip: String,
    pub port: i64,
}

pub struct TrackerSuccessResponse {
    pub interval: i64,
    pub min_interval: Option<i64>,
    pub tracker_id: Option<String>,
    pub complete: i64,
    pub incomplete: i64,
    pub peers: Vec<TrackerResponsePeer>,
}

pub enum TrackerResponse {
    Failure(String),
    Warning(String),
    Success(TrackerSuccessResponse),
}
