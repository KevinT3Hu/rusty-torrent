use torrent_parser::model::TrackerResponse;

pub struct TrackerStatus {
    pub seeders: i64,
    pub leechers: i64,
    pub complete: i64,
    pub incomplete: i64,
}

#[derive(Default)]
pub enum TrackerConnectionState {
    Connected(TrackerStatus),
    Timeout(String),
    #[default]
    NotContacted,
}

pub struct Tracker {
    pub announce: String,
    pub interval: i64,
    pub min_interval: Option<i64>,
    pub state: TrackerConnectionState,
    pub traker_id: Option<String>,
}

impl Tracker {
    pub fn new(announce: String) -> Self {
        Tracker {
            announce,
            interval: 0,
            min_interval: None,
            state: TrackerConnectionState::default(),
            traker_id: None,
        }
    }

    pub fn update(&mut self, resp: &TrackerResponse) {
        match resp {
            TrackerResponse::Success(success) => {
                self.interval = success.interval;
                self.min_interval = success.min_interval;
                if let Some(id) = &success.tracker_id {
                    self.traker_id = Some(id.clone());
                }
                self.state = TrackerConnectionState::Connected(TrackerStatus {
                    seeders: success.complete,
                    leechers: success.incomplete,
                    complete: success.complete,
                    incomplete: success.incomplete,
                });
            }
            TrackerResponse::Failure(msg) => {
                self.state = TrackerConnectionState::Timeout(msg.clone());
            }
            TrackerResponse::Warning(msg) => {
                self.state = TrackerConnectionState::Timeout(msg.clone());
            }
        }
    }
}

impl From<String> for Tracker {
    fn from(value: String) -> Self {
        Tracker::new(value)
    }
}

// we consider two trackers to be equal if they have the same announce URL
impl PartialEq for Tracker {
    fn eq(&self, other: &Self) -> bool {
        self.announce == other.announce
    }
}
