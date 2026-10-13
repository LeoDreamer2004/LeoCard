use super::super::{TexasHoldemAdapter, TexasHoldemSession};
use super::{auto_play, bot};
use leocard_protocol::TexasHoldemEvent;
use leocard_texas_holdem::{Phase, TexasHoldemAction};

impl TexasHoldemSession {
    pub(in super::super) fn play_automatic_action(&mut self) -> Option<Vec<TexasHoldemEvent>> {
        let (player, action) = {
            let game = self.game.as_ref()?;
            let player = game.current_player()?;
            let core_player = game.game().current_player()?;
            let action = if self.room.players[usize::from(player.0)].is_bot {
                bot::Policy::choose(game.game(), core_player, &mut self.bot_decision)?
            } else {
                self.bot_decision.clear();
                auto_play::Policy::choose(game.game(), core_player)?
            };
            (player, action)
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let mut events = self.game.as_mut()?.act(player, action).ok()?;
        events.extend(self.fold_disconnected_players());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_automatic_action_delay_for_current_turn();
        Some(events)
    }

    pub(in super::super) fn fold_disconnected_players(&mut self) -> Vec<TexasHoldemEvent> {
        let mut events = Vec::new();
        while let Some(current) = self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)
        {
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
            let automatic_action = TexasHoldemAction::Fold;
            match game.act(current, automatic_action) {
                Ok(auto_events) => events.extend(auto_events),
                Err(_) => break,
            }
        }
        events
    }
}
