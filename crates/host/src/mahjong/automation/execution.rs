use super::super::{MahjongSession, MahjongWinFeedback};
use super::{auto_play, bot};
use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongActionOutcome, MahjongBotAction};

use leocard_protocol::MahjongEvent;

impl MahjongSession {
    pub(in super::super) fn play_automatic_action(
        &mut self,
        player: PlayerId,
    ) -> Option<(Vec<MahjongEvent>, MahjongActionOutcome)> {
        let action = if self.room.players.get(usize::from(player.0))?.is_bot {
            bot::Policy::choose(self.game.as_ref()?, player, &mut self.bot_decision)?
        } else {
            self.bot_decision.clear();
            auto_play::Policy::choose(self.game.as_ref()?, player)?
        };
        let game = self.game.as_mut()?;
        let public_event = match action {
            MahjongBotAction::Discard(tile) => Some(MahjongEvent::TileDiscarded { player, tile }),
            _ => None,
        };
        let mut events = MahjongWinFeedback::before_discard(game, public_event.as_ref());
        events.extend(public_event);
        let outcome = match action {
            MahjongBotAction::Discard(tile) => game.discard(player, tile),
            MahjongBotAction::Respond(claim) => game.respond_to_claim(player, claim),
            MahjongBotAction::SelfDraw => game.declare_self_draw(player),
            MahjongBotAction::ConcealedKong(tile) => game.declare_concealed_kong(player, tile),
            MahjongBotAction::AddedKong(tile) => game.declare_added_kong(player, tile),
        }
        .ok()?;
        Some((events, outcome))
    }
}
