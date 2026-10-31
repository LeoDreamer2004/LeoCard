use super::super::{AUTOMATIC_ACTION_DELAY, ShengjiSession};
use leocard_game_common::PlayerId;

use leocard_shengji::ShengjiPhase;

impl ShengjiSession {
    pub(in super::super) fn automatic_player(&self) -> Option<PlayerId> {
        if self.presentation.trick.is_some() || self.presentation.throw_failure.is_some() {
            return None;
        }
        let game = self.game.as_ref()?;
        let player = match game.phase() {
            ShengjiPhase::Burying => game.dealer()?,
            ShengjiPhase::BottomCopying => game.bottom_copy()?.current()?,
            ShengjiPhase::BottomCopyBurying => game.bottom_copy()?.bottom_holder()?,
            ShengjiPhase::FiveTrumpCrossing => game
                .five_trump_crossing()?
                .pending_players()
                .into_iter()
                .find(|player| self.is_automatic_participant(*player))?,
            ShengjiPhase::Playing => game.current_player()?,
            _ => return None,
        };
        self.is_automatic_participant(player).then_some(player)
    }

    fn is_automatic_participant(&self, player: PlayerId) -> bool {
        self.room
            .players
            .get(usize::from(player.0))
            .is_some_and(|participant| {
                participant.is_bot
                    || participant.auto_play
                    || !participant.connected
                    || participant.left
            })
    }

    pub(in super::super) fn reset_automatic_action(&mut self) {
        self.flow.automatic_action = self
            .automatic_player()
            .map(|player| (player, AUTOMATIC_ACTION_DELAY));
    }
}
