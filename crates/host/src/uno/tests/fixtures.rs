use crate::ConnectionId;
use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{ClientCommand, ClientMessage, RequestId, RoomId};
use leocard_protocol::{
    JoinRequest, PlayerGameProfiles, ProfileId, ReconnectToken, join_identity_payload,
};

pub(super) const ROOM: RoomId = RoomId(108);
pub(super) const HOST: ConnectionId = ConnectionId(1);

pub(super) fn message(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

pub(super) fn join_command(name: &str, token: u64) -> ClientCommand {
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
