pub(super) use super::super::{
    QiGui523Session, duration_ceil_seconds, from_core_player, record_qigui523_play, to_core_player,
};
pub(super) use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameEvent, JoinRequest, PlayerGameProfiles, PlayerId, ProfileId,
    PublicPlay, QiGui523Event, QiGui523Snapshot, ReconnectToken, RejectReason, RequestId, RoomId,
    SeatId, ServerEvent, join_identity_payload,
};
use leocard_qigui523::{Phase, QiGuiRuleSet, can_beat, classify};
pub(super) use std::collections::HashMap;
use std::io::Cursor;
pub(super) use std::time::Duration;

pub(super) const ROOM: RoomId = RoomId(523);
pub(super) const HOST: ConnectionId = ConnectionId(10);
pub(super) const SECOND: ConnectionId = ConnectionId(20);
pub(super) const THIRD: ConnectionId = ConnectionId(30);

pub(super) fn qigui523_snapshot(event: &ServerEvent) -> Option<&QiGui523Snapshot> {
    let ServerEvent::GameSnapshot(snapshot) = event else {
        return None;
    };
    snapshot.qigui523()
}

pub(super) fn qigui523_play_effect(event: &ServerEvent) -> Option<(PlayerId, &PublicPlay)> {
    let ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect { player, play })) =
        event
    else {
        return None;
    };
    Some((*player, play))
}

pub(super) fn rules() -> QiGuiRuleSet {
    QiGuiRuleSet {
        player_count: 6,
        ..QiGuiRuleSet::default()
    }
}

pub(super) fn message(connection_request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(connection_request), command)
}

pub(super) fn join_command(name: &str, token: ReconnectToken) -> ClientCommand {
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

pub(super) fn join_three(session: &mut QiGui523Session) {
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

pub(super) fn ready_and_start(session: &mut QiGui523Session) -> Vec<Delivery> {
    for connection in [HOST, SECOND, THIRD] {
        session.handle(
            connection,
            message(3, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST, message(4, ClientCommand::StartGame))
}

pub(super) fn send_next(
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

pub(super) fn connection_for_player(session: &QiGui523Session, player: PlayerId) -> ConnectionId {
    session
        .players
        .iter()
        .find(|participant| participant.id == player)
        .expect("player is connected")
        .connection
}

pub(super) fn rejection(deliveries: &[Delivery]) -> Option<&RejectReason> {
    deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::Rejected { reason } => Some(reason),
            _ => None,
        })
}

pub(super) fn avatar_png(width: u32, height: u32) -> Vec<u8> {
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

pub(super) fn finish_game(session: &mut QiGui523Session) {
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
