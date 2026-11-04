use super::RoomSession;
use crate::ConnectionId;
use leocard_protocol::{AvatarFrameId, PlayerViolation, RejectReason};

impl RoomSession {
    pub(crate) fn publish_economy(
        &mut self,
        connection: ConnectionId,
        coins: u32,
        avatar_frame: AvatarFrameId,
    ) -> Result<(), RejectReason> {
        let player = self
            .participants_mut()
            .find(|player| player.connection == connection && !player.left)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if player.game_profiles.coins != coins || player.game_profiles.avatar_frame != avatar_frame
        {
            player.game_profiles.coins = coins;
            player.game_profiles.avatar_frame = avatar_frame;
            self.bump_revision();
        }
        Ok(())
    }
}
