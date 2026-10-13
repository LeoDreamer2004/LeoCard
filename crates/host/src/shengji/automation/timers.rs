use super::super::{AUTOMATIC_ACTION_DELAY, ShengjiSession};
use leocard_shengji::{Phase, ShengjiPlayerId};

impl ShengjiSession {
    pub(in super::super) fn automatic_player(&self) -> Option<ShengjiPlayerId> {
        if self.presentation.trick.is_some() || self.presentation.throw_failure.is_some() {
            return None;
        }
        let game = self.game.as_ref()?;
        let player = match game.phase() {
            Phase::Burying => game.dealer()?,
            Phase::BottomCopying => game.bottom_copy()?.current()?,
            Phase::BottomCopyBurying => game.bottom_copy()?.bottom_holder()?,
            Phase::FiveTrumpCrossing => game
                .five_trump_crossing()?
                .pending_players()
                .into_iter()
                .find(|player| self.is_automatic_participant(*player))?,
            Phase::Playing => game.current_player()?,
            _ => return None,
        };
        self.is_automatic_participant(player).then_some(player)
    }

    fn is_automatic_participant(&self, player: ShengjiPlayerId) -> bool {
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
