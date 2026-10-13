use super::super::action::AutomaticUnoAction;
use leocard_uno::{GameState, PendingSwap};

pub(in super::super::super) struct Policy;

impl Policy {
    pub(in super::super) fn should_call_uno(game: &GameState, forced_skip: bool) -> bool {
        game.turn()
            .is_some_and(|turn| turn.pending_swap.is_none() && !forced_skip)
    }

    pub(in super::super) fn choose(
        game: &GameState,
        forced_skip: bool,
    ) -> Option<AutomaticUnoAction> {
        let turn = game.turn()?;
        let player = turn.current_player;
        let action = if forced_skip {
            AutomaticUnoAction::ResolveSkip
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
                    AutomaticUnoAction::SwapOneTarget { target, index }
                }
                PendingSwap::SwapOneGive { .. } => {
                    let card = game
                        .player(player)?
                        .hand()
                        .iter()
                        .copied()
                        .max_by_key(|card| card.score())?;
                    AutomaticUnoAction::GiveSwapOneCard(card)
                }
                PendingSwap::ForceTrade { .. } => {
                    let second = game
                        .players()
                        .iter()
                        .filter(|candidate| candidate.id() != player && !candidate.eliminated())
                        .min_by_key(|candidate| candidate.hand().len())?
                        .id();
                    AutomaticUnoAction::ForceTrade {
                        first: player,
                        second,
                    }
                }
                PendingSwap::ChooseColor { .. } => {
                    AutomaticUnoAction::ChooseColor(Self::preferred_color(game, player))
                }
                PendingSwap::SevenSwap { .. } => {
                    let target = game
                        .players()
                        .iter()
                        .filter(|candidate| candidate.id() != player && !candidate.eliminated())
                        .min_by_key(|candidate| candidate.hand().len())?
                        .id();
                    AutomaticUnoAction::SevenSwap(target)
                }
                PendingSwap::ColorRoulette { .. } => {
                    AutomaticUnoAction::ChooseColor(Self::preferred_color(game, player))
                }
            }
        } else if turn.current_color.is_none() {
            AutomaticUnoAction::ChooseColor(Self::preferred_color(game, player))
        } else if turn.pending_kind.is_some() {
            if let Some(card) = game
                .player(player)?
                .hand()
                .iter()
                .copied()
                .find(|card| game.can_play(player, *card))
            {
                let color = Self::automatic_chosen_color(game, player, card);
                AutomaticUnoAction::Play { card, color }
            } else {
                AutomaticUnoAction::AcceptDrawPenalty
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
                AutomaticUnoAction::Play { card, color: None }
            } else {
                AutomaticUnoAction::ResolveSkip
            }
        } else if let Some(card) = turn.drawn_card {
            let color = Self::automatic_chosen_color(game, player, card);
            AutomaticUnoAction::Play { card, color }
        } else if let Some(card) = game
            .player(player)?
            .hand()
            .iter()
            .copied()
            .find(|card| game.can_play(player, *card))
        {
            let color = Self::automatic_chosen_color(game, player, card);
            AutomaticUnoAction::Play { card, color }
        } else {
            AutomaticUnoAction::Draw
        };
        Some(action)
    }
}
