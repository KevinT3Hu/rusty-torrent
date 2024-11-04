use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    sync::{Arc, RwLock},
};

pub struct TcpConnectionManager {
    connections: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    peer_id: String,
    info_hash: Vec<u8>,
}

impl TcpConnectionManager {
    pub fn new(peer_id: String, info_hash: Vec<u8>) -> Self {
        TcpConnectionManager {
            connections: Default::default(),
            peer_id,
            info_hash,
        }
    }

    pub fn connect_to_peer(&self, addr: SocketAddr) {
        let stream = TcpStream::connect(addr).unwrap();
        self.connections.write().unwrap().insert(addr, stream);
    }

    pub fn listen(&self) {}
}
