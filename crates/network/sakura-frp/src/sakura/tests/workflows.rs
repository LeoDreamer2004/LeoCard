use super::super::tunnel::WssLink;
use super::fixture::{ApiFixture, Step};
use serde_json::{Value, json};

fn parent() -> Value {
    json!({
        "id": 1, "name": "Room", "node": 7, "type": "tcp", "online": true,
        "status": 0, "status_reason": null, "remote": "37897",
        "local_ip": "127.0.0.2", "local_port": 52300,
    })
}

fn creation_steps() -> Vec<Step> {
    vec![
        Step::new("GET", "/tunnels", 200, json!([parent()])),
        Step::new(
            "GET",
            "/user/info",
            200,
            json!({"id": 1, "name": "User", "avatar": "", "tunnels": 2}),
        ),
        Step::new(
            "GET",
            "/nodes",
            200,
            json!({"7": {"name": "Node", "host": "example.test", "vip": 0, "flag": 4}}),
        ),
        Step::new(
            "GET",
            "/domain/get",
            200,
            json!({
                "domain": "sample.nyat.app", "words": ["www"], "records": [],
            }),
        ),
        Step::new(
            "POST",
            "/tunnels",
            201,
            json!({"id": 2, "name": "Room_WSS", "remote": "23001"}),
        ),
    ]
}

#[test]
fn create_wss_copies_target_and_binds_a_certificate_domain() {
    let mut steps = creation_steps();
    steps.push(Step::new("POST", "/domain/bind", 204, Value::Null));
    let api = ApiFixture::new(steps);
    assert_eq!(api.client.create_wss(1).unwrap().id, 2);
    let bodies = api.finish();
    let create: Value = serde_json::from_str(&bodies[4]).unwrap();
    assert_eq!(create["node"], 7);
    assert_eq!(create["local_ip"], "127.0.0.2");
    assert_eq!(create["local_port"], 52300);
    assert_eq!(create["type"], "tcp");
    assert_eq!(
        create["extra"],
        "auto_https = auto\nauto_https_mode = passthrough"
    );
    let link = WssLink::from_note(create["note"].as_str().unwrap()).unwrap();
    assert_eq!(link.source, 1);
    assert_eq!(link.title, "Room");
    assert_eq!(
        bodies[5],
        "rr=www&tunnel=2&domain=sample.nyat.app&type=CNAME"
    );
}

#[test]
fn failed_binding_deletes_only_the_new_wss_tunnel() {
    let mut steps = creation_steps();
    steps.push(Step::new(
        "POST",
        "/domain/bind",
        400,
        json!({"msg": "binding denied"}),
    ));
    steps.push(Step::new(
        "POST",
        "/tunnel/delete",
        200,
        json!({"deleted": [2], "failed": []}),
    ));
    let api = ApiFixture::new(steps);
    let error = api.client.create_wss(1).unwrap_err();
    assert!(error.contains("已撤销"));
    assert!(error.contains("binding denied"));
    let bodies = api.finish();
    let delete: Value = serde_json::from_str(&bodies[6]).unwrap();
    assert_eq!(delete, json!({"ids": "2"}));
}

#[test]
fn repeated_creation_returns_existing_child_without_consuming_quota() {
    let mut child = parent();
    child["id"] = json!(2);
    child["name"] = json!("Room_WSS");
    child["note"] = json!(
        WssLink {
            source: 1,
            title: "Room".to_owned()
        }
        .note()
        .unwrap()
    );
    let api = ApiFixture::new(vec![Step::new(
        "GET",
        "/tunnels",
        200,
        json!([parent(), child]),
    )]);
    assert_eq!(api.client.create_wss(1).unwrap().id, 2);
    assert_eq!(api.finish().len(), 1);
}
