use super::super::McGuessSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameSnapshot, JoinRequest, McGuessSnapshot, PlayerGameProfiles,
    ProfileId, ReconnectToken, RequestId, RoomId, ServerEvent,
};

pub(super) fn send(
    session: &mut McGuessSession,
    connection: u64,
    command: ClientCommand,
) -> Vec<Delivery> {
    let connection = ConnectionId(connection);
    let next = session
        .room
        .last_requests
        .get(&connection)
        .map_or(1, |request| request.0 + 1);
    session.handle(
        connection,
        ClientMessage::new(RoomId(99), RequestId(next), command),
    )
}

pub(super) fn join(session: &mut McGuessSession, connection: u64) {
    let key = SigningKey::from_bytes(&[connection as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{connection}"),
        reconnect_token: ReconnectToken(connection),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    let deliveries = send(session, connection, ClientCommand::join(request));
    assert!(
        deliveries
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Joined { .. }))
    );
}

pub(super) fn snapshot(deliveries: &[Delivery], connection: u64) -> &McGuessSnapshot {
    deliveries
        .iter()
        .find_map(|delivery| {
            if delivery.recipient == ConnectionId(connection)
                && let ServerEvent::GameSnapshot(GameSnapshot::McGuess(snapshot)) =
                    &delivery.message.event
            {
                Some(snapshot)
            } else {
                None
            }
        })
        .expect("recipient receives a guessing snapshot")
}
