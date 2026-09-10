mod adapter;
mod settlement;

use super::*;
use crate::ConnectionId;
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, GameCommand, GameSnapshot, JoinRequest, PROTOCOL_VERSION, PlayerGameProfiles,
    ProfileId, ReconnectToken, RequestId, Revision, SeatId, ServerEvent, ServerMessage,
    TexasHoldemCommand, TexasHoldemViolation, decode_frame, encode_frame, join_identity_payload,
};
use leocard_protocol::{
    ClientMessage, MatchId, PlayerId, RoomId, TexasHoldemEvent, TexasHoldemPhaseView,
};
use leocard_texas_holdem::TexasHoldemCard;
use leocard_texas_holdem::{
    Phase, TexasHoldemAction, TexasHoldemBlindKind, TexasHoldemRank, TexasHoldemRuleSet,
    TexasHoldemStreet, TexasHoldemSuit, build_deck,
};
use std::collections::HashMap;

const ROOM: RoomId = RoomId(9527);
const HOST_CONNECTION: ConnectionId = ConnectionId(10);
const SECOND_CONNECTION: ConnectionId = ConnectionId(20);
const THIRD_CONNECTION: ConnectionId = ConnectionId(30);

fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join_command(name: &str, token: u64) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.to_be_bytes());
    secret[8] = 9;
    let key = SigningKey::from_bytes(&secret);
    let reconnect_token = ReconnectToken(token);
    let profile_id = ProfileId(key.verifying_key().to_bytes());
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token,
        profile_id,
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}

fn session_waiting_for_blinds() -> TexasHoldemSession {
    let mut session = TexasHoldemSession::new(
        ROOM,
        52300,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    for (connection, name, token) in [
        (HOST_CONNECTION, "房主", 1),
        (SECOND_CONNECTION, "玩家二", 2),
        (THIRD_CONNECTION, "玩家三", 3),
    ] {
        session.handle(connection, message(1, join_command(name, token)));
    }
    for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    let deliveries = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
    )));
    session
}

fn started_session() -> TexasHoldemSession {
    let mut session = session_waiting_for_blinds();
    for _ in 0..2 {
        let player = session.game().unwrap().current_player().unwrap();
        session
            .game
            .as_mut()
            .unwrap()
            .act(player, TexasHoldemAction::PostBlind)
            .unwrap();
    }
    session
}

fn connection_for(session: &TexasHoldemSession, player: PlayerId) -> ConnectionId {
    session
        .room
        .players
        .iter()
        .find(|participant| participant.id == player)
        .unwrap()
        .connection
}
