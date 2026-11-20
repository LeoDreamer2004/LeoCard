use super::super::domains::DomainSettings;
use super::super::tunnel::{Tunnel, WssLink};
use serde_json::json;
use std::slice;

#[test]
fn wss_address_uses_bound_certificate_domain_and_source_title() {
    let note = WssLink {
        source: 1,
        title: "Room".to_owned(),
    }
    .note()
    .unwrap();
    let mut tunnel: Tunnel = serde_json::from_value(json!({
        "id": 2, "name": "Room_WSS", "node": 7, "type": "tcp", "online": false,
        "status": 0, "status_reason": null, "note": note,
        "extra": "auto_https = auto\nauto_https_mode = passthrough", "remote": "23001",
        "local_ip": "127.0.0.1", "local_port": 52300,
    }))
    .unwrap();
    tunnel.load_wss_link();
    assert_eq!(tunnel.protocol_label(), "WSS");
    assert_eq!(tunnel.title(), "Room");
    assert!(!tunnel.can_create_wss());
    assert_eq!(tunnel.address(&[]), None);
    let domains: DomainSettings = serde_json::from_value(json!({
        "domain": "sample.nyat.app", "records": [{ "rr": "game", "type": "CNAME", "tunnel": 2 }],
    }))
    .unwrap();
    domains.assign(slice::from_mut(&mut tunnel));
    assert_eq!(
        tunnel.address(&[]).as_deref(),
        Some("wss://game.sample.nyat.app:23001/")
    );
}

#[test]
fn domain_allocation_preserves_existing_bindings_and_uses_allowed_words() {
    let domains: DomainSettings = serde_json::from_value(json!({
        "domain": "sample.nyat.app", "words": ["www"],
        "records": [
            { "rr": "www", "type": "CNAME", "tunnel": 1 },
            { "rr": "www-www", "type": "CNAME", "tunnel": 2 },
        ],
    }))
    .unwrap();
    assert_eq!(domains.available_record().unwrap(), "www-www-www");
}
