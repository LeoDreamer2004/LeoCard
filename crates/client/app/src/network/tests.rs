use super::{NETWORK_ROOM_ID, normalize_server_address};
use crate::{ClientModel, ClientPhaseRef};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    AvatarId, GameKind, GameRules, LobbyPlayerView, LobbySnapshot, PROTOCOL_VERSION,
    PlayerGameProfiles, PlayerGender, PlayerPublicInfo, ProfileId, Revision,
    RoomModerationSnapshot, RoomParticipant, SeatId, ServerEvent, ServerMessage,
};

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

#[test]
fn spectator_interaction_controls_follow_authoritative_room_updates() {
    let mut model = ClientModel::new(NETWORK_ROOM_ID);
    let mut lobby = snapshot();
    if let ServerEvent::LobbySnapshot(snapshot) = &mut lobby.event {
        snapshot.players.push(LobbyPlayerView {
            player: PlayerPublicInfo {
                id: PlayerId(0),
                profile_id: ProfileId([1; 32]),
                name: "房主".to_owned(),
                avatar: None,
                reference_points: 0,
                completed_games: 0,
                game_profiles: Default::default(),
            },
            seat: Some(SeatId(0)),
            ready: true,
            connected: true,
        });
    }
    model.apply(message(ServerEvent::Joined { you: PlayerId(128) }));
    model.apply(lobby.clone());
    let mut room = RoomModerationSnapshot {
        host: Some(PlayerId(0)),
        spectator_interactions_blocked: true,
        participants: vec![RoomParticipant {
            player: PlayerPublicInfo {
                id: PlayerId(128),
                profile_id: ProfileId([2; 32]),
                name: "旁观者".to_owned(),
                avatar: Some(AvatarId(7)),
                reference_points: 100,
                completed_games: 3,
                game_profiles: PlayerGameProfiles {
                    gender: PlayerGender::Female,
                    ..Default::default()
                },
            },
            spectating: true,
            connected: true,
            is_bot: false,
        }],
    };
    model.apply(message(ServerEvent::RoomModerationSnapshot(room.clone())));
    assert_eq!(model.player_name(PlayerId(128)), Some("旁观者"));
    assert_eq!(model.player_avatar(PlayerId(128)), Some(AvatarId(7)));
    assert_eq!(
        model.local_game_profiles().unwrap().gender,
        PlayerGender::Female
    );
    assert!(!model.is_host());
    assert!(!model.can_send_interactions());
    model.apply(lobby);
    assert!(!model.can_send_interactions());
    room.spectator_interactions_blocked = false;
    model.apply(message(ServerEvent::RoomModerationSnapshot(room.clone())));
    assert!(model.can_send_interactions());
    room.spectator_interactions_blocked = true;
    model.apply(message(ServerEvent::RoomModerationSnapshot(room)));
    model.apply(message(ServerEvent::Joined { you: PlayerId(0) }));
    assert!(model.is_host());
    assert!(model.can_send_interactions());
}

#[test]
fn kick_announcements_and_the_kicked_clients_exit_have_distinct_state() {
    let mut remaining = ClientModel::new(NETWORK_ROOM_ID);
    remaining.apply(message(ServerEvent::Joined { you: PlayerId(0) }));
    remaining.apply(snapshot());
    remaining.apply(message(ServerEvent::PlayerKicked {
        name: "访客".to_owned(),
    }));
    assert_eq!(remaining.last_notice(), Some("访客已被房主踢出"));
    assert!(matches!(remaining.phase(), ClientPhaseRef::Lobby(_)));
    assert!(!remaining.kicked_from_room());

    let mut kicked = remaining;
    kicked.apply(message(ServerEvent::KickedFromRoom));
    assert!(kicked.kicked_from_room());
    assert!(kicked.left_room());
    assert!(!kicked.room_closed());
    assert!(matches!(kicked.phase(), ClientPhaseRef::Closed));
}

fn message(event: ServerEvent) -> ServerMessage {
    ServerMessage {
        action_timing: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: NETWORK_ROOM_ID,
        revision: Revision(1),
        in_reply_to: None,
        game_context: None,
        event,
    }
}

fn snapshot() -> ServerMessage {
    message(ServerEvent::LobbySnapshot(LobbySnapshot {
        game: GameKind::QiGui523,
        rules: GameRules::QiGui523(Default::default()),
        host_port: 52300,
        host: Some(PlayerId(0)),
        players: Vec::new(),
        entrance: None,
    }))
}
