use semver::Version;
use std::path::PathBuf;

#[derive(Debug)]
pub enum UpdateEvent {
    Available {
        version: Version,
        total: Option<u64>,
    },
    Progress {
        downloaded: u64,
        total: Option<u64>,
    },
    UpToDate {
        version: Version,
    },
    Ready {
        version: Version,
        staged: PathBuf,
    },
    Failed(String),
}
