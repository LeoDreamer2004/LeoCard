use super::RoomSession;
use crate::ConnectionId;
use leocard_protocol::{PlayerViolation, RejectReason};

impl RoomSession {
    pub(crate) fn publish_coins(
        &mut self,
        connection: ConnectionId,
        coins: u32,
    ) -> Result<(), RejectReason> {
        let player = self
            .players
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if player.game_profiles.coins != coins {
            player.game_profiles.coins = coins;
            self.bump_revision();
        }
        Ok(())
    }
}
