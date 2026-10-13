use super::super::{MahjongSession, MahjongWinFeedback, to_core_player};
use super::action::AutomaticMahjongAction;
use super::{auto_play, bot};
use leocard_mahjong::ActionOutcome;
use leocard_protocol::{MahjongEvent, PlayerId};

impl MahjongSession {
    pub(in super::super) fn play_automatic_action(
        &mut self,
        player: PlayerId,
    ) -> Option<(Vec<MahjongEvent>, ActionOutcome)> {
        let core_player = to_core_player(player);
        let action = if self.room.players.get(usize::from(player.0))?.is_bot {
            bot::Policy::choose(self.game.as_ref()?, core_player, &mut self.bot_decision)?
        } else {
            self.bot_decision.clear();
            auto_play::Policy::choose(self.game.as_ref()?, core_player)?
        };
        let game = self.game.as_mut()?;
        let public_event = match action {
            AutomaticMahjongAction::Discard(tile) => {
                Some(MahjongEvent::TileDiscarded { player, tile })
            }
            _ => None,
        };
        let mut events = MahjongWinFeedback::before_discard(game, public_event.as_ref());
        events.extend(public_event);
        let outcome = match action {
            AutomaticMahjongAction::Discard(tile) => game.discard(core_player, tile),
            AutomaticMahjongAction::Respond(claim) => game.respond_to_claim(core_player, claim),
            AutomaticMahjongAction::SelfDraw => game.declare_self_draw(core_player),
            AutomaticMahjongAction::ConcealedKong(tile) => {
                game.declare_concealed_kong(core_player, tile)
            }
            AutomaticMahjongAction::AddedKong(tile) => game.declare_added_kong(core_player, tile),
        }
        .ok()?;
        Some((events, outcome))
    }
}
