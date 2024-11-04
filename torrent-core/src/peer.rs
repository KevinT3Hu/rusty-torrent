use torrent_parser::model::TrackerResponsePeer;

pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub am_choking: bool,
    pub am_interested: bool,
    pub peer_choking: bool,
    pub peer_interested: bool,
    pub bitfield: Vec<u8>,
}

impl PartialEq for Peer {
    // we consider two peers equal if they have the same id or ip&port
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id || (self.ip == other.ip && self.port == other.port)
    }
}

impl From<TrackerResponsePeer> for Peer {
    fn from(peer: TrackerResponsePeer) -> Self {
        Peer {
            id: peer.peer_id.unwrap_or_default(),
            ip: peer.ip,
            port: peer.port as u16,
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
            bitfield: vec![0; 0],
        }
    }
}
