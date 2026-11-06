use super::super::{TexasAdapter, TexasSession};
use super::{auto_play, bot};
use leocard_protocol::TexasEvent;
use leocard_texas_holdem::{TexasAction, TexasPhase};

impl TexasSession {
    pub(in super::super) fn play_automatic_action(&mut self) -> Option<Vec<TexasEvent>> {
        let game = self.game.as_ref()?;
        let player = game.current_player()?;
        let core_player = game.game().current_player()?;
        let is_bot = self
            .room
            .players
            .iter()
            .find(|participant| participant.id == player)?
            .is_bot;
        let voting = matches!(game.game().phase(), TexasPhase::ChoosingRunout);
        let action = if voting {
            self.bot_decision.clear();
            None
        } else if is_bot {
            Some(bot::Policy::choose(
                game.game(),
                core_player,
                &mut self.bot_decision,
            )?)
        } else {
            self.bot_decision.clear();
            Some(auto_play::Policy::choose(game.game(), core_player)?)
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), TexasPhase::Complete(_)));
        let game = self.game.as_mut()?;
        let mut events = if voting {
            game.choose_runout(player, is_bot).ok()?
        } else {
            game.act(player, action?).ok()?
        };
        events.extend(self.resolve_disconnected_turns());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_automatic_action_delay_for_current_turn();
        Some(events)
    }

    pub(in super::super) fn resolve_disconnected_turns(&mut self) -> Vec<TexasEvent> {
        let mut events = Vec::new();
        while let Some(current) = self.game.as_ref().and_then(TexasAdapter::current_player) {
            let disconnected = self
                .room
                .players
                .iter()
                .find(|player| player.id == current)
                .is_some_and(|player| (!player.connected && !player.is_bot) || player.left);
            if !disconnected {
                break;
            }
            let Some(game) = self.game.as_mut() else {
                break;
            };
            if game.game().blind_to_post().is_some() {
                break;
            }
            let result = if matches!(game.game().phase(), TexasPhase::ChoosingRunout) {
                game.choose_runout(current, false)
            } else {
                game.act(current, TexasAction::Fold)
            };
            match result {
                Ok(auto_events) => events.extend(auto_events),
                Err(_) => break,
            }
        }
        events
    }
}
