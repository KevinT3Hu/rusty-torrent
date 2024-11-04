use std::{cell::RefCell, sync::Arc, time::Duration};

use reqwest::Client;
use tokio::{spawn, sync::RwLock, task::JoinHandle, time::sleep};
use torrent_parser::{
    model::{TorrentMetadata, TrackerResponse},
    parse_tracker_response,
};

use crate::{
    peer::Peer,
    tracker::{Tracker, TrackerConnectionState},
};

pub struct ManagedTorrent {
    pub metadata: TorrentMetadata,
    pub name: String,
    pub location: String,
    pub peers: Arc<RwLock<Vec<Peer>>>,
    pub trackers: Vec<Arc<RwLock<Tracker>>>,
    task_handles: RefCell<Vec<JoinHandle<()>>>,
    pub downloaded: Arc<RwLock<u64>>,
    pub uploaded: Arc<RwLock<u64>>,
    client: Arc<Client>,
    peer_id: String,
    port: u32,
}

impl ManagedTorrent {
    pub fn from_torrent_metadata(
        metadata: TorrentMetadata,
        name: Option<String>,
        location: String,
        peer_id: String,
        port: u32,
        client: Arc<Client>,
    ) -> Self {
        let meta_name = metadata.info.name.clone();
        let tracker = metadata.announce.clone();
        let trackers_extra = metadata
            .announce_list
            .clone()
            .map(|trackers| trackers.into_iter().flatten().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut trackers: Vec<Tracker> = vec![tracker]
            .into_iter()
            .chain(trackers_extra)
            .map(Tracker::from)
            .collect();
        trackers.dedup();
        let trackers = trackers
            .into_iter()
            .map(|x| Arc::new(RwLock::new(x)))
            .collect();

        let name = name.unwrap_or(meta_name);
        ManagedTorrent {
            metadata,
            name,
            location,
            downloaded: Default::default(),
            uploaded: Default::default(),
            peers: Default::default(),
            trackers,
            task_handles: Default::default(),
            client,
            peer_id,
            port,
        }
    }

    pub fn start(&self) {
        // spawn a job to contact trackers every interval
        for tracker in &self.trackers {
            let client = Arc::clone(&self.client);
            let tracker = Arc::clone(tracker);
            let peers = Arc::clone(&self.peers);
            let downloaded = Arc::clone(&self.downloaded);
            let uploaded = Arc::clone(&self.uploaded);
            let info_hash = self.metadata.info_hash.clone();
            let peer_id = self.peer_id.clone();
            let port = self.port;
            // spawn a job to contact tracker
            let handle = spawn(async move {
                loop {
                    let r_tracker = tracker.read().await;
                    let interval = r_tracker.interval;
                    drop(r_tracker);
                    sleep(Duration::from_secs(interval as u64)).await;
                    let mut tracker = tracker.write().await;
                    let downloaded = downloaded.read().await;
                    let uploaded = uploaded.read().await;
                    let mut req = client
                        .get(&tracker.announce)
                        .timeout(Duration::from_secs(10))
                        .query(&[("info_hash", &info_hash)])
                        .query(&[("peer_id", &peer_id)])
                        .query(&[("port", &port)])
                        .query(&[("compact", &1)])
                        .query(&[("uploaded", &*downloaded), ("downloaded", &*uploaded)]);

                    if let Some(id) = &tracker.traker_id {
                        req = req.query(&[("trackerid", id)]);
                    }

                    if matches!(tracker.state, TrackerConnectionState::NotContacted) {
                        req = req.query(&[("event", "started")]);
                    }

                    let resp = req.send().await;
                    match resp {
                        Err(e) => {
                            tracker.state =
                                crate::tracker::TrackerConnectionState::Timeout(e.to_string());
                        }
                        Ok(resp) => {
                            let body = resp.bytes().await.unwrap().to_vec();
                            let parsed = parse_tracker_response(body);
                            if let Ok(parsed) = parsed {
                                tracker.update(&parsed);

                                if let TrackerResponse::Success(resp) = parsed {
                                    let mut peers = peers.write().await;
                                    for peer in resp.peers {
                                        peers.push(Peer::from(peer));
                                    }
                                    peers.dedup();
                                }
                            }
                        }
                    }
                }
            });
            self.task_handles.borrow_mut().push(handle);
        }
    }
}

impl Drop for ManagedTorrent {
    fn drop(&mut self) {
        for handle in self.task_handles.borrow_mut().drain(..) {
            handle.abort();
        }
        // send stopped event to trackers
        for tracker in &self.trackers {
            let client = Arc::clone(&self.client);
            let tracker = Arc::clone(tracker);
            let info_hash = self.metadata.info_hash.clone();
            let peer_id = self.peer_id.clone();
            let port = self.port;
            spawn(async move {
                let tracker = tracker.read().await;
                let _ = client
                    .get(&tracker.announce)
                    .query(&[("info_hash", &info_hash)])
                    .query(&[("peer_id", &peer_id)])
                    .query(&[("port", &port)])
                    .query(&[("event", "stopped")])
                    .send()
                    .await;
            });
        }
    }
}
