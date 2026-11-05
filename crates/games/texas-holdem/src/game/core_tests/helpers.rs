use crate::{
    TexasAction, TexasActionOutcome, TexasCard, TexasGameState, TexasHandResult, TexasPhase,
    TexasRuleSet, TexasStreet, build_deck,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

pub(super) fn game(stacks: &[u32], voting: bool) -> TexasGameState {
    let rules = TexasRuleSet {
        player_count: stacks.len() as u8,
        side_pots: false,
        run_it_twice: voting,
        ..Default::default()
    };
    TexasGameState::new_with_stacks(
        rules,
        PlayerId(stacks.len() as u8 - 1),
        stacks.to_vec(),
        build_deck(false),
    )
    .unwrap()
}

pub(super) fn act(game: &mut TexasGameState, action: TexasAction) -> TexasActionOutcome {
    game.act(game.current_player().unwrap(), action).unwrap()
}

pub(super) fn blinds(game: &mut TexasGameState) {
    act(game, TexasAction::PostBlind);
    act(game, TexasAction::PostBlind);
}

pub(super) fn to_street(game: &mut TexasGameState, street: TexasStreet, preflop_target: u32) {
    blinds(game);
    if preflop_target > 2 {
        act(game, TexasAction::RaiseTo(preflop_target));
    }
    while *game.phase() != TexasPhase::Betting(street) {
        let player = game.current_player().unwrap();
        let action = if game.amount_to_call(player).unwrap() > 0 {
            TexasAction::Call
        } else {
            TexasAction::Check
        };
        act(game, action);
    }
}

pub(super) fn result(game: &TexasGameState) -> &TexasHandResult {
    let TexasPhase::Complete(result) = game.phase() else {
        panic!("hand not complete: {:?}", game.phase())
    };
    result
}

pub(super) fn vote_all(game: &mut TexasGameState, twice: bool) -> TexasActionOutcome {
    let mut outcome = None;
    while matches!(game.phase(), TexasPhase::ChoosingRunout) {
        outcome = Some(
            game.choose_runout(game.current_player().unwrap(), twice)
                .unwrap(),
        );
    }
    outcome.unwrap()
}

pub(super) fn ordered_deck(prefix: &[TexasCard]) -> Vec<TexasCard> {
    let used = prefix.iter().copied().collect::<HashSet<_>>();
    assert_eq!(used.len(), prefix.len());
    prefix
        .iter()
        .copied()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !used.contains(card)),
        )
        .collect()
}
