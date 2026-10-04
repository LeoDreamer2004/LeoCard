use super::{
    PendingDrawReveal, UnoSession, analysis_events, append_finished_event, automatic_chosen_color,
    events_for_outcome, from_core_player, pending_draw_reveal_for_outcome, preferred_color,
};
use crate::{AUTO_PLAY_DELAY, AutoPlayDelayState};
use leocard_protocol::{PlayerId, UnoEvent};
use leocard_uno::{PendingSwap, UnoActionContext};

impl UnoSession {
    pub(super) fn automatically_resolves_skip(&self) -> bool {
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

    pub(super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let current = self
            .game
            .as_ref()?
            .turn()
            .map(|turn| from_core_player(turn.current_player))?;
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

    pub(super) fn reset_auto_play_delay(&mut self) {
        self.auto_play_delay = self
            .current_automatic_player()
            .map(|player| AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
    }

    pub(super) fn play_automatic_action(
        &mut self,
    ) -> Option<(Vec<UnoEvent>, Option<PendingDrawReveal>)> {
        let forced_skip = self.automatically_resolves_skip();
        let game = self.game.as_mut()?;
        let turn = game.turn()?;
        let player = turn.current_player;
        let uno_before = UnoActionContext::capture(game, player);
        let uno_outcome = (turn.pending_swap.is_none() && !forced_skip)
            .then(|| game.call_uno(player).ok())
            .flatten();
        let mut events = uno_outcome
            .as_ref()
            .map(|outcome| events_for_outcome(outcome, &[]))
            .unwrap_or_default();
        if let (Some(before), Some(outcome)) = (uno_before, uno_outcome.as_ref()) {
            events.extend(analysis_events(
                self.statistics
                    .as_mut()
                    .expect("active UNO games have a recorder")
                    .observe(before, outcome, &[], false, game),
            ));
        }
        let before = UnoActionContext::capture(game, player)?;
        let mut played = Vec::new();
        let outcome = if forced_skip {
            game.resolve_skip(player)
        } else if let Some(pending) = turn.pending_swap {
            match pending {
                PendingSwap::SwapOneTarget { .. } => {
                    let target = game
                        .players()
                        .iter()
                        .filter(|candidate| candidate.id() != player && !candidate.eliminated())
                        .max_by_key(|candidate| candidate.hand().len())?;
                    let target = target.id();
                    let index = fastrand::usize(..game.player(target)?.hand().len());
                    game.choose_swap_one_target(player, target, index)
                }
                PendingSwap::SwapOneGive { .. } => {
                    let card = game
                        .player(player)?
                        .hand()
                        .iter()
                        .copied()
                        .max_by_key(|card| card.score())?;
                    game.give_swap_one_card(player, card)
                }
                PendingSwap::ForceTrade { .. } => {
                    let second = game
                        .players()
                        .iter()
                        .filter(|candidate| candidate.id() != player && !candidate.eliminated())
                        .min_by_key(|candidate| candidate.hand().len())?
                        .id();
                    game.force_trade_hands(player, player, second)
                }
                PendingSwap::ChooseColor { .. } => {
                    game.choose_initial_color(player, preferred_color(game, player))
                }
                PendingSwap::SevenSwap { .. } => {
                    let target = game
                        .players()
                        .iter()
                        .filter(|candidate| candidate.id() != player && !candidate.eliminated())
                        .min_by_key(|candidate| candidate.hand().len())?
                        .id();
                    game.choose_seven_swap_target(player, target)
                }
                PendingSwap::ColorRoulette { .. } => {
                    game.choose_initial_color(player, preferred_color(game, player))
                }
            }
        } else if turn.current_color.is_none() {
            game.choose_initial_color(player, preferred_color(game, player))
        } else if turn.pending_kind.is_some() {
            if let Some(card) = game
                .player(player)?
                .hand()
                .iter()
                .copied()
                .find(|card| game.can_play(player, *card))
            {
                let color = automatic_chosen_color(game, player, card);
                played.push((card, color));
                game.play_card(player, card, color)
            } else {
                game.accept_draw_penalty(player)
            }
        } else if turn.pending_skip > 0 || turn.skipped_turns_remaining > 0 {
            if turn.skipped_turns_remaining == 0
                && let Some(card) = game
                    .player(player)?
                    .hand()
                    .iter()
                    .copied()
                    .find(|card| game.can_play(player, *card))
            {
                played.push((card, None));
                game.play_card(player, card, None)
            } else {
                game.resolve_skip(player)
            }
        } else if let Some(card) = turn.drawn_card {
            let color = automatic_chosen_color(game, player, card);
            played.push((card, color));
            game.play_card(player, card, color)
        } else if let Some(card) = game
            .player(player)?
            .hand()
            .iter()
            .copied()
            .find(|card| game.can_play(player, *card))
        {
            let color = automatic_chosen_color(game, player, card);
            played.push((card, color));
            game.play_card(player, card, color)
        } else {
            game.draw_card(player)
        }
        .ok()?;
        events.extend(events_for_outcome(&outcome, &played));
        append_finished_event(&mut events, game);
        events.extend(analysis_events(
            self.statistics
                .as_mut()
                .expect("active UNO games have a recorder")
                .observe(before, &outcome, &played, false, game),
        ));
        let draw_reveal = pending_draw_reveal_for_outcome(&outcome, game);
        Some((events, draw_reveal))
    }
}
