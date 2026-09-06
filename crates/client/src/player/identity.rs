use std::io;

use ed25519_dalek::{Signer, SigningKey};
use leocard_protocol::{
    ClientCommand, JoinRequest, PlayerGameProfiles, ProfileId, ReconnectToken, RoomId,
    join_identity_payload,
};

#[derive(Clone)]
pub struct PlayerIdentity {
    signing_key: SigningKey,
}

impl PlayerIdentity {
    pub fn generate() -> io::Result<Self> {
        let mut secret = [0; 32];
        getrandom::fill(&mut secret)
            .map_err(|error| io::Error::other(format!("无法生成玩家身份：{error}")))?;
        Ok(Self::from_secret_bytes(secret))
    }

    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&secret),
        }
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn profile_id(&self) -> ProfileId {
        ProfileId(self.signing_key.verifying_key().to_bytes())
    }

    pub(crate) fn join_command(
        &self,
        room_id: RoomId,
        name: &str,
        reconnect_token: ReconnectToken,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> ClientCommand {
        let payload = join_identity_payload(
            room_id,
            reconnect_token,
            name,
            reference_points,
            completed_games,
            &game_profiles,
        );
        ClientCommand::join(JoinRequest {
            name: name.to_owned(),
            reconnect_token,
            profile_id: self.profile_id(),
            reference_points,
            completed_games,
            game_profiles,
            identity_signature: self.signing_key.sign(&payload).to_bytes().to_vec(),
        })
    }
}
