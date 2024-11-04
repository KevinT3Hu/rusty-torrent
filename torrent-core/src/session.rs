use std::{collections::HashMap, path::Path, sync::Arc};

use reqwest::Client;
use tokio::sync::RwLock;
use torrent_parser::parse_torrent_file;
use uuid::Uuid;

use crate::{
    error::{RustyTorrentError, RustyTorrentResult},
    torrent::ManagedTorrent,
};

pub type TorrentId = Uuid;

pub struct RustyTorrentSession {
    torrents: RwLock<HashMap<TorrentId, ManagedTorrent>>,
    http_client: Arc<Client>,
    default_location: String,
    peer_id: String,
    port: u32,
}

impl RustyTorrentSession {
    pub fn new(
        default_location: String,
        specifier: &[char; 2],
        version: &[char; 4],
        port: u32,
    ) -> Self {
        // generate a 20-byte peer ID
        // get current process id and time for randomness
        let pid = std::process::id();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // generate 12 byte string based on pid and time
        let pid_time = format!("{:x}{:x}", pid, time);
        let pid_time = &pid_time[..12];

        let peer_id = format!(
            "-{}{}-{}",
            specifier.iter().collect::<String>(),
            version.iter().collect::<String>(),
            pid_time
        );

        RustyTorrentSession {
            torrents: RwLock::new(HashMap::new()),
            http_client: Default::default(),
            default_location,
            peer_id,
            port,
        }
    }

    pub async fn add_torrent(
        &self,
        torrent_path: String,
        name: Option<String>,
        location: Option<String>,
        start: bool,
    ) -> RustyTorrentResult<()> {
        let location = location.unwrap_or(self.default_location.clone());
        let mut torrents = self.torrents.write().await;
        let meta = parse_torrent_file(&torrent_path)?;
        let torrent = ManagedTorrent::from_torrent_metadata(
            meta,
            name,
            location,
            self.peer_id.clone(),
            self.port,
            Arc::clone(&self.http_client),
        );
        let id = Uuid::new_v4();
        torrents.insert(id, torrent);
        if start {
            self.start_torrent(id).await?;
        }

        Ok(())
    }

    pub async fn start_torrent(&self, id: TorrentId) -> RustyTorrentResult<()> {
        let torrents = self.torrents.read().await;
        let torrent = torrents
            .get(&id)
            .ok_or(RustyTorrentError::TorrentNotFound(id))?;

        let download_dir = Path::new(&torrent.location);
        if !download_dir.exists() {
            std::fs::create_dir_all(download_dir)?;
        }

        let torrent_path = download_dir.join(&torrent.name);

        torrent.start();

        Ok(())
    }
}
