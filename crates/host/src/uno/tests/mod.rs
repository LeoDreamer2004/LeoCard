mod jump_in;
mod lifecycle;
mod profile;
mod snapshots;

use super::*;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, GameSnapshot, GameViolation, PlayerId,
    RejectReason, RequestId, RoomId, ServerEvent, UnoCommand, UnoProfileStats, UnoSnapshot,
    UnoViolation,
};
use leocard_protocol::{
    JoinRequest, PlayerGameProfiles, ProfileId, ReconnectToken, SeatId, TexasHoldemCommand,
    join_identity_payload,
};
use leocard_uno::Mode;
use leocard_uno::{
    ActionOutcome, GameState, Phase, UnoCard, UnoChallengeResult, UnoColor, UnoPlayerId,
    UnoRuleSet, build_deck_for_rules,
};
#[cfg(test)]
use leocard_uno::{build_deck, build_no_mercy_deck};
use std::time::Duration;

const ROOM: RoomId = RoomId(108);
const HOST: ConnectionId = ConnectionId(1);

fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join_command(name: &str, token: u64) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.to_be_bytes());
    secret[8] = 7;
    let key = SigningKey::from_bytes(&secret);
    let reconnect_token = ReconnectToken(token);
    let game_profiles = PlayerGameProfiles::default();
    let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
    ClientCommand::join(JoinRequest {
        name: name.to_owned(),
        reconnect_token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles,
        identity_signature: key.sign(&payload).to_bytes().to_vec(),
    })
}
