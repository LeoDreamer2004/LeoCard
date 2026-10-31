use super::super::{QiGui523Session, duration_ceil_seconds};
use crate::{AUTOMATIC_ACTION_DELAY, AutomaticActionDelayState};
use leocard_game_common::PlayerId;
use leocard_protocol::QiGui523TurnTimerView;
use leocard_qigui523::QiGuiPhase;
use std::time::Duration;

#[derive(Clone, Debug)]
pub(in super::super) struct TurnTimerState {
    pub(in super::super) player: PlayerId,
    pub(in super::super) base_remaining: Duration,
    pub(in super::super) reserve_remaining: Vec<Duration>,
}

impl QiGui523Session {
    pub(in super::super) fn initialize_turn_timer(&mut self) {
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(game) = self.game.as_ref() else {
            self.turn_timer = None;
            return;
        };
        let Some(current) = game.trick().map(|trick| trick.current_player()) else {
            self.turn_timer = None;
            return;
        };
        let control = self.rules.time_control;
        self.turn_timer = Some(TurnTimerState {
            player: current,
            base_remaining: Duration::from_secs(u64::from(control.base_seconds())),
            reserve_remaining: vec![
                Duration::from_secs(u64::from(control.reserve_seconds()));
                game.players().len()
            ],
        });
    }

    pub(in super::super) fn reset_timer_for_current_turn(&mut self) {
        self.reset_automatic_action_delay_for_current_turn();
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(current) = self.game.as_ref().and_then(|game| {
            matches!(game.phase(), QiGuiPhase::Playing)
                .then(|| game.trick().map(|trick| trick.current_player()))
                .flatten()
        }) else {
            self.turn_timer = None;
            return;
        };
        let base = Duration::from_secs(u64::from(self.rules.time_control.base_seconds()));
        match self.turn_timer.as_mut() {
            Some(timer) => {
                timer.player = current;
                timer.base_remaining = base;
            }
            None => self.initialize_turn_timer(),
        }
    }

    pub(in super::super) fn turn_timer_view(&self) -> Option<QiGui523TurnTimerView> {
        let timer = self.turn_timer.as_ref()?;
        Some(QiGui523TurnTimerView {
            player: timer.player,
            base_seconds: duration_ceil_seconds(timer.base_remaining),
            reserve_seconds: duration_ceil_seconds(
                timer.reserve_remaining[usize::from(timer.player.0)],
            ),
        })
    }

    pub(in super::super) fn current_player(&self) -> Option<PlayerId> {
        self.game.as_ref().and_then(|game| {
            matches!(game.phase(), QiGuiPhase::Playing)
                .then(|| game.trick().map(|trick| trick.current_player()))
                .flatten()
        })
    }

    pub(in super::super) fn current_player_is_disconnected(&self) -> bool {
        let Some(current) = self.current_player() else {
            return false;
        };
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| !player.connected && !player.is_bot)
    }

    pub(in super::super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let current = self.current_player()?;
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| player.is_bot || (player.connected && player.auto_play))
            .then_some(current)
    }

    pub(in super::super) fn reset_automatic_action_delay_for_current_turn(&mut self) {
        self.automatic_action_delay =
            self.current_automatic_player()
                .map(|player| AutomaticActionDelayState {
                    player,
                    remaining: AUTOMATIC_ACTION_DELAY,
                });
    }
}
