use super::super::{MahjongSession, from_core_player};
use leocard_mahjong::{MahjongPlayerId, Phase};
use leocard_protocol::PlayerId;

impl MahjongSession {
    pub(in super::super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let game = self.game.as_ref()?;
        let automatic = |player: MahjongPlayerId| {
            let player = from_core_player(player);
            self.room
                .players
                .iter()
                .find(|participant| participant.id == player && !participant.left)
                .is_some_and(|participant| participant.is_bot || participant.auto_play)
                .then_some(player)
        };
        match game.phase() {
            Phase::Playing => automatic(game.current_player()),
            Phase::WaitingForClaims(pending) => {
                pending.waiting_for().into_iter().find_map(automatic)
            }
            Phase::Dealing { .. } | Phase::ReplacingFlower { .. } | Phase::Finished(_) => None,
        }
    }
}
