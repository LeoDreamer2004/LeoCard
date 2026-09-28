use super::download::parse_sha256;

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
