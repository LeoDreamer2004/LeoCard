use ed25519_dalek::{Signer, SigningKey};
use leocard_game_common::PlayerId;
use leocard_host::{ConnectionId, Delivery, GameSetup, HostSession};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameRules, GameViolation, JoinRequest, LobbySnapshot,
    PlayerGameProfiles, PlayerViolation, ProfileId, ReconnectToken, RejectReason, RequestId,
    RoomId, RoomViolation, SeatId, ServerEvent,
};

const ROOM: RoomId = RoomId(314);

fn send(
    session: &mut HostSession,
    connection: u64,
    request: u64,
    command: ClientCommand,
) -> Vec<Delivery> {
    session.handle(
        ConnectionId(connection),
        ClientMessage::new(ROOM, RequestId(request), command),
    )
}

fn joined_room(rules: GameRules) -> HostSession {
    let mut session = HostSession::new(ROOM, GameSetup::shuffled(52300, rules)).unwrap();
    for connection in 1..=3_u8 {
        let key = SigningKey::from_bytes(&[connection; 32]);
        let mut request = JoinRequest {
            name: format!("玩家{connection}"),
            reconnect_token: ReconnectToken(u64::from(connection)),
            profile_id: ProfileId(key.verifying_key().to_bytes()),
            reference_points: 123,
            completed_games: 10,
            game_profiles: PlayerGameProfiles::default(),
            identity_signature: Vec::new(),
        };
        request.identity_signature = key
            .sign(&request.identity_payload(ROOM))
            .to_bytes()
            .to_vec();
        accepted(&send(
            &mut session,
            u64::from(connection),
            1,
            ClientCommand::join(request),
        ));
    }
    session
}

fn accepted(deliveries: &[Delivery]) {
    assert!(
        deliveries
            .iter()
            .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. })),
        "{deliveries:?}"
    );
}

fn lobby(deliveries: &[Delivery]) -> &LobbySnapshot {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(lobby) => Some(lobby),
            _ => None,
        })
        .expect("lobby snapshot")
}

fn rejection(deliveries: &[Delivery], reason: RejectReason) {
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0].message.event,
        ServerEvent::Rejected { reason }
    );
}

#[test]
fn every_game_kicks_the_target_and_notifies_all_remaining_lobby_players() {
    for rules in [
        GameRules::Uno(Default::default()),
        GameRules::Shengji(Default::default()),
        GameRules::Mahjong(Default::default()),
        GameRules::Texas(Default::default()),
        GameRules::QiGui523(Default::default()),
        GameRules::McGuess(Default::default()),
        GameRules::Blackjack(Default::default()),
        GameRules::Doudizhu(Default::default()),
    ] {
        let mut session = joined_room(rules);
        let before = send(&mut session, 1, 2, ClientCommand::RequestSnapshot);
        let seat = lobby(&before)
            .players
            .iter()
            .find(|player| player.player.id == PlayerId(1))
            .unwrap()
            .seat
            .unwrap();
        let deliveries = send(
            &mut session,
            1,
            3,
            ClientCommand::KickPlayer {
                player: PlayerId(1),
            },
        );
        accepted(&deliveries);
        let target: Vec<_> = deliveries
            .iter()
            .filter(|delivery| delivery.recipient == ConnectionId(2))
            .collect();
        assert_eq!(target.len(), 1);
        assert_eq!(target[0].message.event, ServerEvent::KickedFromRoom);
        for connection in [1, 3] {
            assert!(deliveries.iter().any(|delivery| delivery.recipient == ConnectionId(connection) && matches!(&delivery.message.event, ServerEvent::PlayerKicked { name } if name == "玩家2")));
        }
        let current = lobby(&deliveries);
        assert_eq!(current.host, Some(PlayerId(0)));
        assert_eq!(current.players.len(), 2);
        assert!(
            current
                .players
                .iter()
                .all(|player| player.player.id != PlayerId(1))
        );
        assert!(
            current
                .players
                .iter()
                .all(|player| player.player.reference_points == 123)
        );
        rejection(
            &send(&mut session, 2, 2, ClientCommand::SetReady { ready: true }),
            RejectReason::Player(PlayerViolation::NotJoined),
        );
        let replaced = send(
            &mut session,
            1,
            4,
            ClientCommand::ConfigureBotSeat {
                seat,
                occupied: true,
            },
        );
        accepted(&replaced);
        assert_eq!(lobby(&replaced).players.len(), 3);
    }
}

#[test]
fn kicking_requires_the_host_and_cannot_target_the_host_or_a_bot() {
    let mut session = joined_room(GameRules::Uno(Default::default()));
    rejection(
        &send(
            &mut session,
            3,
            2,
            ClientCommand::KickPlayer {
                player: PlayerId(1),
            },
        ),
        RejectReason::Room(RoomViolation::OnlyHostCanKick),
    );
    rejection(
        &send(
            &mut session,
            1,
            2,
            ClientCommand::KickPlayer {
                player: PlayerId(0),
            },
        ),
        RejectReason::Room(RoomViolation::CannotKickPlayer),
    );
    let before = send(&mut session, 1, 3, ClientCommand::RequestSnapshot);
    let empty = (0..8)
        .map(SeatId)
        .find(|seat| {
            lobby(&before)
                .players
                .iter()
                .all(|player| player.seat != Some(*seat))
        })
        .unwrap();
    let added = send(
        &mut session,
        1,
        4,
        ClientCommand::ConfigureBotSeat {
            seat: empty,
            occupied: true,
        },
    );
    let bot = lobby(&added)
        .players
        .iter()
        .find(|player| player.player.profile_id == ProfileId([0; 32]))
        .unwrap()
        .player
        .id;
    rejection(
        &send(
            &mut session,
            1,
            5,
            ClientCommand::KickPlayer { player: bot },
        ),
        RejectReason::Room(RoomViolation::CannotKickPlayer),
    );
    let after = send(&mut session, 1, 6, ClientCommand::RequestSnapshot);
    assert_eq!(lobby(&after).players.len(), 4);
}

#[test]
fn kicking_is_rejected_once_the_game_has_started() {
    let mut session = joined_room(GameRules::Uno(Default::default()));
    for connection in [2, 3] {
        accepted(&send(
            &mut session,
            connection,
            2,
            ClientCommand::SetReady { ready: true },
        ));
    }
    accepted(&send(&mut session, 1, 2, ClientCommand::StartGame));
    rejection(
        &send(
            &mut session,
            1,
            3,
            ClientCommand::KickPlayer {
                player: PlayerId(1),
            },
        ),
        RejectReason::Game(GameViolation::GameAlreadyStarted),
    );
}
