use super::state::normalize_server_address;
use super::{LocalPlayerConnection, NETWORK_ROOM_ID, NetworkState, TcpGameClient};
use crate::ClientModel;
use leocard_protocol::{ClientCommand, PlayerId, SeatId};
use leocard_qigui523::QiGuiRuleSet;
use std::time::{Duration, Instant};

#[test]
fn join_address_accepts_dns_names_and_ip_literals() {
    assert_eq!(
        normalize_server_address("frp-off.com:52436").unwrap(),
        "frp-off.com:52436"
    );
    assert_eq!(
        normalize_server_address(" 192.168.1.8:52300 ").unwrap(),
        "192.168.1.8:52300"
    );
    assert_eq!(
        normalize_server_address("[2001:db8::1]:52300").unwrap(),
        "[2001:db8::1]:52300"
    );
}

#[test]
fn join_address_rejects_missing_or_invalid_ports() {
    for address in [
        "frp-off.com",
        "frp-off.com:0",
        "frp-off.com:70000",
        ":52300",
    ] {
        assert!(normalize_server_address(address).is_err(), "{address}");
    }
}
fn unused_local_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for(client: &mut TcpGameClient, predicate: impl Fn(&ClientModel) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        client.poll();
        if predicate(client.model()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out; network state is {:?}", client.state());
}

#[test]
fn real_tcp_host_and_two_joiners_reach_the_same_three_player_game() {
    let port = unused_local_port();
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let mut host = TcpGameClient::host_with_profile(
        port,
        rules.into(),
        LocalPlayerConnection::temporary("房主", None).unwrap(),
    )
    .unwrap();
    wait_for(&mut host, |model| model.lobby().is_some());

    let address = format!("127.0.0.1:{port}");
    let mut guest_a = TcpGameClient::join("甲", &address).unwrap();
    let mut guest_b = TcpGameClient::join("乙", &address).unwrap();
    wait_for(&mut host, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });
    wait_for(&mut guest_a, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });
    wait_for(&mut guest_b, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });

    assert_eq!(host.model().you(), Some(PlayerId(0)));
    assert_eq!(guest_a.model().room_id(), NETWORK_ROOM_ID);
    assert_eq!(guest_b.model().room_id(), NETWORK_ROOM_ID);
    assert_eq!(host.model().host_port(), Some(port));
    assert_eq!(guest_a.model().host_port(), Some(port));
    assert_eq!(guest_b.model().host_port(), Some(port));

    host.send(ClientCommand::SelectSeat { seat: SeatId(0) });
    guest_a.send(ClientCommand::SelectSeat { seat: SeatId(2) });
    guest_b.send(ClientCommand::SelectSeat { seat: SeatId(5) });
    host.send(ClientCommand::SetReady { ready: true });
    guest_a.send(ClientCommand::SetReady { ready: true });
    guest_b.send(ClientCommand::SetReady { ready: true });
    wait_for(&mut host, |model| {
        model
            .lobby()
            .is_some_and(|lobby| lobby.players.iter().all(|player| player.ready))
    });
    host.send(ClientCommand::StartGame);
    wait_for(&mut host, |model| model.qigui523_game().is_some());
    wait_for(&mut guest_a, |model| model.qigui523_game().is_some());
    wait_for(&mut guest_b, |model| model.qigui523_game().is_some());
    assert_eq!(guest_a.model().qigui523_game().unwrap().host_port, port);
    assert_eq!(
        host.model().qigui523_game().unwrap().your_hand.len(),
        usize::from(rules.hand_size)
    );
}

#[test]
fn host_close_room_event_stops_guests_without_reconnecting() {
    let port = unused_local_port();
    let mut host = TcpGameClient::host_with_profile(
        port,
        QiGuiRuleSet::default().into(),
        LocalPlayerConnection::temporary("房主", None).unwrap(),
    )
    .unwrap();
    wait_for(&mut host, |model| model.lobby().is_some());

    let mut guest = TcpGameClient::join("访客", &format!("127.0.0.1:{port}")).unwrap();
    wait_for(&mut guest, |model| model.lobby().is_some());
    wait_for(&mut host, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 2)
    });

    assert!(host.send(ClientCommand::CloseRoom));
    wait_for(&mut host, ClientModel::room_closed);
    wait_for(&mut guest, ClientModel::room_closed);

    assert!(!matches!(host.state(), NetworkState::Reconnecting(_)));
    assert!(!matches!(guest.state(), NetworkState::Reconnecting(_)));
}
