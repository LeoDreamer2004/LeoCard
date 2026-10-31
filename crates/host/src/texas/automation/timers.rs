use super::super::TexasSession;
use crate::{AUTOMATIC_ACTION_DELAY, AutomaticActionDelayState};
use leocard_game_common::PlayerId;

use leocard_texas_holdem::TexasBlindKind;
use std::time::Duration;

const FIRST_SMALL_BLIND_POST_DELAY: Duration = Duration::from_millis(1800);
const BLIND_POST_DELAY: Duration = Duration::from_millis(900);

impl TexasSession {
    pub(in super::super) fn current_automatic_action_player(&self) -> Option<PlayerId> {
        let game = self.game.as_ref()?;
        let current = game.current_player()?;
        if game.game().blind_to_post().is_some() {
            return Some(current);
        }
        self.room
            .players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| {
                !player.left && (player.is_bot || (player.connected && player.auto_play))
            })
            .then_some(current)
    }

    pub(in super::super) fn automatic_delay_for_current_turn(&self) -> Duration {
        let Some(game) = self.game.as_ref() else {
            return AUTOMATIC_ACTION_DELAY;
        };
        match game.game().blind_to_post() {
            Some((_, TexasBlindKind::Small, _)) if game.game().hand_number() == 0 => {
                FIRST_SMALL_BLIND_POST_DELAY
            }
            Some(_) => BLIND_POST_DELAY,
            None => AUTOMATIC_ACTION_DELAY,
        }
    }

    pub(in super::super) fn reset_automatic_action_delay_for_current_turn(&mut self) {
        let remaining = self.automatic_delay_for_current_turn();
        self.automatic_action_delay = self
            .current_automatic_action_player()
            .map(|player| AutomaticActionDelayState { player, remaining });
    }
}
