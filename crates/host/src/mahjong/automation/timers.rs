use super::super::MahjongSession;
use leocard_game_common::PlayerId;
use leocard_mahjong::MahjongPhase;

impl MahjongSession {
    pub(in super::super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let game = self.game.as_ref()?;
        let automatic = |player: PlayerId| {
            self.room
                .players
                .iter()
                .find(|participant| participant.id == player && !participant.left)
                .is_some_and(|participant| participant.is_bot || participant.auto_play)
                .then_some(player)
        };
        match game.phase() {
            MahjongPhase::Playing => automatic(game.current_player()),
            MahjongPhase::WaitingForClaims(pending) => {
                pending.waiting_for().into_iter().find_map(automatic)
            }
            MahjongPhase::Dealing { .. }
            | MahjongPhase::ReplacingFlower { .. }
            | MahjongPhase::Finished(_) => None,
        }
    }
}
