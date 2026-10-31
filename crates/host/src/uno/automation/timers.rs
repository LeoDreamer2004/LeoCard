use super::super::UnoSession;
use crate::{AUTOMATIC_ACTION_DELAY, AutomaticActionDelayState};
use leocard_game_common::PlayerId;

impl UnoSession {
    pub(in super::super) fn automatically_resolves_skip(&self) -> bool {
        !self.rules.action_stacking_enabled()
            && self
                .game
                .as_ref()
                .and_then(|game| game.turn())
                .is_some_and(|turn| {
                    turn.pending_swap.is_none()
                        && turn.current_color.is_some()
                        && turn.pending_kind.is_none()
                        && (turn.pending_skip > 0 || turn.skipped_turns_remaining > 0)
                })
    }

    pub(in super::super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let current = self.game.as_ref()?.turn().map(|turn| turn.current_player)?;
        self.room
            .players
            .iter()
            .find(|player| player.id == current && !player.left)
            .is_some_and(|player| {
                self.automatically_resolves_skip()
                    || player.auto_play
                    || !player.connected
                    || player.is_bot
            })
            .then_some(current)
    }

    pub(in super::super) fn reset_automatic_action_delay(&mut self) {
        self.automatic_action_delay =
            self.current_automatic_player()
                .map(|player| AutomaticActionDelayState {
                    player,
                    remaining: AUTOMATIC_ACTION_DELAY,
                });
    }
}
