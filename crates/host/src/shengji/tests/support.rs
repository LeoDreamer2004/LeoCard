#[cfg(feature = "developer")]
pub(super) use super::super::{AUTOMATIC_ACTION_DELAY, PLAYER_COUNT};
pub(super) use super::super::{
    BIDDING_GRACE, BOTTOM_COPY_DECISION_TIMEOUT, BOTTOM_FLIP_HOLD_DURATION,
    BOTTOM_FLIP_START_DELAY, DEAL_INTERVAL, POWER_OUTAGE_BIDDING_GRACE, ShengjiSession,
    THROW_FAILURE_RETURN_DURATION, THROW_FAILURE_SHOW_DURATION, finished_reference_point_magnitude,
};
pub(super) use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameSnapshot, JoinRequest, PlayerGameProfiles, ProfileId,
    ReconnectToken, RequestId, RoomId, SeatId, ServerEvent, ShengjiSnapshot, join_identity_payload,
};
use leocard_shengji::{ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit, build_deck};
pub(super) use std::time::Duration;

pub(super) const ROOM: RoomId = RoomId(8080);

pub(super) fn message(connection_index: u8, request: u64, command: ClientCommand) -> ClientMessage {
    let _ = connection_index;
    ClientMessage::new(ROOM, RequestId(request), command)
}

pub(super) fn join_command(index: u8) -> ClientCommand {
    let name = format!("玩家{index}");
    let token = ReconnectToken(u64::from(index) + 1);
    let mut secret = [0; 32];
    secret[0] = index + 1;
    let key = SigningKey::from_bytes(&secret);
    let game_profiles = PlayerGameProfiles::default();
    let signature = key
        .sign(&join_identity_payload(
            ROOM,
            token,
            &name,
            0,
            0,
            &game_profiles,
        ))
        .to_bytes()
        .to_vec();
    ClientCommand::join(JoinRequest {
        name,
        reconnect_token: token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: signature,
    })
}

pub(super) fn started_session() -> (ShengjiSession, [ConnectionId; 4], ShengjiCard) {
    started_session_with_rules(ShengjiRuleSet::default())
}

pub(super) fn started_session_with_rules(
    rules: ShengjiRuleSet,
) -> (ShengjiSession, [ConnectionId; 4], ShengjiCard) {
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let mut deck = build_deck();
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(0, index);
    let (session, connections) = started_session_with_deck(rules, deck);
    (session, connections, target)
}

pub(super) fn started_session_with_deck(
    rules: ShengjiRuleSet,
    deck: Vec<ShengjiCard>,
) -> (ShengjiSession, [ConnectionId; 4]) {
    let mut session = ShengjiSession::new(ROOM, 52300, rules, deck).unwrap();
    let connections = [
        ConnectionId(10),
        ConnectionId(20),
        ConnectionId(30),
        ConnectionId(40),
    ];
    for (index, connection) in connections.into_iter().enumerate() {
        session.handle(
            connection,
            message(index as u8, 1, join_command(index as u8)),
        );
        session.handle(
            connection,
            message(
                index as u8,
                2,
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ),
        );
        session.handle(
            connection,
            message(index as u8, 3, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(connections[0], message(0, 4, ClientCommand::StartGame));
    (session, connections)
}

pub(super) fn game_snapshot(deliveries: &[Delivery], recipient: ConnectionId) -> ShengjiSnapshot {
    deliveries
        .iter()
        .rev()
        .find_map(|delivery| {
            if delivery.recipient != recipient {
                return None;
            }
            let ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) =
                &delivery.message.event
            else {
                return None;
            };
            Some(snapshot.clone())
        })
        .expect("recipient receives a Shengji snapshot")
}
