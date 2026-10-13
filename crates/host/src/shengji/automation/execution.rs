use super::super::ShengjiSession;
use super::{auto_play, bot};
use leocard_shengji::{ActionOutcome, ShengjiPlayerId};

impl ShengjiSession {
    pub(in super::super) fn play_automatic_action(
        &mut self,
        player: ShengjiPlayerId,
    ) -> Option<ActionOutcome> {
        let is_bot = self.room.players.get(usize::from(player.0))?.is_bot;
        let game = self.game.as_mut()?;
        if is_bot {
            bot::Policy::act(game, player, &mut self.bot_decision)
        } else {
            self.bot_decision.clear();
            auto_play::Policy::act(game, player)
        }
    }
}
