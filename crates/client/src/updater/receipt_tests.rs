use super::{UpdateReceipt, read_completed, sidecar};
use leocard_protocol::ProfileId;
use std::{env, fs, path::PathBuf};

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Self {
        let path =
            env::temp_dir().join(format!("leocard-update-receipt-{:016x}", fastrand::u64(..)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn only_completed_upgrades_belonging_to_the_installed_version_and_profile_count() {
    let directory = TemporaryDirectory::new();
    let target = directory.0.join("leocard");
    let staged = directory.0.join("download");
    let profile = ProfileId([1; 32]);
    let pending = UpdateReceipt::stage(&staged, "9999.0.0", profile).unwrap();
    assert!(!read_completed(&target, profile, "9999.0.0").unwrap());
    UpdateReceipt::complete(&pending, &target).unwrap();
    assert!(!pending.exists());
    assert!(read_completed(&target, profile, "9999.0.0").unwrap());
    assert!(!read_completed(&target, ProfileId([2; 32]), "9999.0.0").unwrap());
    assert!(!read_completed(&target, profile, "9998.0.0").unwrap());
    // Keep the completed marker available if achievement persistence needs a retry.
    assert!(read_completed(&target, profile, "9999.0.0").unwrap());
    let next = UpdateReceipt::stage(&staged, "9999.1.0", profile).unwrap();
    UpdateReceipt::complete(&next, &target).unwrap();
    assert!(read_completed(&target, profile, "9999.1.0").unwrap());
}

#[test]
fn non_upgrades_and_invalid_receipts_cannot_report_a_completed_upgrade() {
    let directory = TemporaryDirectory::new();
    let target = directory.0.join("leocard");
    let staged = directory.0.join("download");
    let profile = ProfileId([1; 32]);
    for version in [env!("CARGO_PKG_VERSION"), "0.0.0", "invalid"] {
        assert!(UpdateReceipt::stage(&staged, version, profile).is_err());
    }
    assert!(!sidecar(&staged, ".receipt").exists());
    for bytes in [vec![], vec![0], vec![1, 255]] {
        fs::write(sidecar(&target, ".update-completed"), bytes).unwrap();
        assert!(read_completed(&target, profile, "9999.0.0").is_err());
    }
}
