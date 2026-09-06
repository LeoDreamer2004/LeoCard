use super::*;
use leocard_protocol::{
    AVATAR_DIMENSION, ClientCommand, JoinRequest, PlayerGameProfiles, ProfileId, QUICK_VOICE_COUNT,
    ReconnectToken, RuleViolation, join_identity_payload,
};

use leocard_qigui523::{QiGuiRank, QiGuiSuit, can_beat, classify};
use std::collections::HashMap;
use std::io::Cursor;

mod automation;
mod lifecycle;
mod lobby;
mod profile;
mod protocol;

const ROOM: RoomId = RoomId(523);
const HOST: ConnectionId = ConnectionId(10);
const SECOND: ConnectionId = ConnectionId(20);
const THIRD: ConnectionId = ConnectionId(30);

fn qigui523_snapshot(event: &ServerEvent) -> Option<&QiGui523Snapshot> {
    let ServerEvent::GameSnapshot(snapshot) = event else {
        return None;
    };
    snapshot.qigui523()
}

fn qigui523_play_effect(event: &ServerEvent) -> Option<(PlayerId, &PublicPlay)> {
    let ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect { player, play })) =
        event
    else {
        return None;
    };
    Some((*player, play))
}

fn rules() -> QiGuiRuleSet {
    QiGuiRuleSet {
        player_count: 6,
        ..QiGuiRuleSet::default()
    }
}

fn message(connection_request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(connection_request), command)
}

fn join_command(name: &str, token: ReconnectToken) -> ClientCommand {
    use ed25519_dalek::{Signer, SigningKey};

    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.0.to_be_bytes());
    secret[8] = 1;
    let key = SigningKey::from_bytes(&secret);
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token: token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}

fn join_three(session: &mut QiGui523Session) {
    for (seat, (connection, name)) in [(HOST, "房主"), (SECOND, "玩家二"), (THIRD, "玩家三")]
        .into_iter()
        .enumerate()
    {
        session.handle(
            connection,
            message(1, join_command(name, ReconnectToken(connection.0))),
        );
        session.handle(
            connection,
            message(
                2,
                ClientCommand::SelectSeat {
                    seat: SeatId(seat as u8),
                },
            ),
        );
    }
}

fn ready_and_start(session: &mut QiGui523Session) -> Vec<Delivery> {
    for connection in [HOST, SECOND, THIRD] {
        session.handle(
            connection,
            message(3, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST, message(4, ClientCommand::StartGame))
}

#[cfg(feature = "developer")]
#[test]
fn developer_player_can_replace_only_their_own_hand() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let player = session.player_id(SECOND).unwrap();
    let other = session.player_id(THIRD).unwrap();
    let other_hand = session
        .game()
        .unwrap()
        .player(to_core_player(other))
        .unwrap()
        .hand()
        .to_vec();
    let replacement = vec![
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(9, QiGuiSuit::Spade, QiGuiRank::Seven),
    ];

    let deliveries = send_next(
        &mut session,
        SECOND,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: replacement.clone(),
        })),
    );

    assert_eq!(deliveries.len(), 3);
    let hand = session
        .game()
        .unwrap()
        .player(to_core_player(player))
        .unwrap()
        .hand();
    assert_eq!(hand.len(), replacement.len());
    assert_eq!(
        hand.iter().filter(|card| **card == replacement[0]).count(),
        2
    );
    assert!(hand.contains(&replacement[2]));
    assert_eq!(
        session
            .game()
            .unwrap()
            .player(to_core_player(other))
            .unwrap()
            .hand(),
        other_hand
    );
}

#[cfg(feature = "developer")]
#[test]
fn developer_player_can_play_a_semantic_pair_from_impossible_physical_copies() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let current = from_core_player(session.game().unwrap().trick().unwrap().current_player());
    let connection = connection_for_player(&session, current);
    let pair = vec![
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(1, QiGuiSuit::Club, QiGuiRank::Joker),
    ];

    send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: pair.clone(),
        })),
    );
    let deliveries = send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: pair,
        })),
    );

    assert!(
        deliveries
            .iter()
            .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert!(matches!(
        session.game().unwrap().phase(),
        Phase::Finished(_)
    ));
}

#[cfg(not(feature = "developer"))]
#[test]
fn normal_build_rejects_developer_hand_commands() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let rejected = send_next(
        &mut session,
        SECOND,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: vec![QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Seven)],
        })),
    );
    assert_eq!(
        rejection(&rejected),
        Some(&RejectReason::DeveloperFeatureUnavailable)
    );
}

fn send_next(
    session: &mut QiGui523Session,
    connection: ConnectionId,
    command: ClientCommand,
) -> Vec<Delivery> {
    let request = session
        .last_requests
        .get(&connection)
        .map_or(1, |request| request.0 + 1);
    session.handle(connection, message(request, command))
}

fn connection_for_player(session: &QiGui523Session, player: PlayerId) -> ConnectionId {
    session
        .players
        .iter()
        .find(|participant| participant.id == player)
        .expect("player is connected")
        .connection
}

fn rejection(deliveries: &[Delivery]) -> Option<&RejectReason> {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::Rejected { reason } => Some(reason),
            _ => None,
        })
}

fn avatar_png(width: u32, height: u32) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
        width,
        height,
        image::Rgba([30, 120, 200, 255]),
    ));
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    output.into_inner()
}

fn finish_game(session: &mut QiGui523Session) {
    for _ in 0..1_000 {
        if matches!(session.game().unwrap().phase(), Phase::Finished(_)) {
            return;
        }
        let (current, playable) = {
            let game = session.game().unwrap();
            let trick = game.trick().unwrap();
            let current = trick.current_player();
            let winning_play = trick.winning_play();
            let playable = game
                .player(current)
                .unwrap()
                .hand()
                .iter()
                .copied()
                .find(|card| {
                    let candidate = classify(&[*card], game.rules()).unwrap();
                    winning_play.is_none_or(|winning| can_beat(&candidate, winning, game.rules()))
                });
            (current, playable)
        };
        let game = session.game.as_mut().unwrap();
        if let Some(card) = playable {
            game.play_cards(current, &[card]).unwrap();
        } else {
            game.pass(current).unwrap();
        }
    }
    panic!("deterministic game did not terminate");
}
