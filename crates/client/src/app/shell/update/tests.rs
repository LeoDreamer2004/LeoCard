use super::download::parse_sha256;
use super::view::update_display;
use super::*;

#[test]
fn parses_standard_sha256sum_output() {
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert_eq!(
        parse_sha256(&format!("{hash}  leocard-linux-x86_64\n")),
        Ok(hash.to_owned())
    );
}

#[test]
fn rejects_malformed_sha256() {
    assert!(parse_sha256("1234  leocard").is_err());
    assert!(parse_sha256(&format!("{}g", "0".repeat(63))).is_err());
}

#[test]
fn progress_display_is_clamped() {
    let state = UpdateState::Downloading {
        version: Version::new(1, 2, 3),
        downloaded: 150,
        total: Some(100),
    };
    let (_, detail, fraction) = update_display(&state);
    assert_eq!(fraction, 1.0);
    assert!(detail.contains("100%"));
}
