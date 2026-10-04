use super::installer::{backup_path, is_helper_path};
use std::path::{Path, PathBuf};

#[test]
fn backup_name_preserves_the_executable_name() {
    assert_eq!(
        backup_path(Path::new("C:/games/leocard.exe")),
        PathBuf::from("C:/games/leocard.exe.old")
    );
    assert_eq!(
        backup_path(Path::new("/opt/leocard")),
        PathBuf::from("/opt/leocard.old")
    );
}

#[test]
fn only_internal_updater_names_are_self_cleaned() {
    assert!(!is_helper_path(Path::new("leocard.exe")));
    assert!(is_helper_path(Path::new("leocard-updater-123.exe")));
}
