use super::fixtures::position;
use crate::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, FourPlayerBot,
    FourPlayerObservation, build_deck,
};
use leocard_game_common::{PlayerId, SearchBudget};
use std::time::Duration;

#[test]
fn prefers_finishing_and_leaves_a_short_teammates_lead_alone() {
    let game = position([&[1, 1], &[0, 8], &[2, 9], &[3, 10]], 0, 0);
    let input = FourPlayerObservation::from_game(&game).unwrap();
    let mut finished = game;
    finished
        .act(PlayerId(0), FourPlayerBot::fallback(&input))
        .unwrap();
    assert_eq!(finished.phase, DoudizhuPhase::Finished);

    let mut game = position([&[10, 10], &[1, 2], &[3, 7, 7], &[6, 6, 8]], 1, 0);
    let card = game.hand(PlayerId(1)).unwrap()[0];
    game.act(PlayerId(1), DoudizhuAction::Play(vec![card]))
        .unwrap();
    let input = FourPlayerObservation::from_game(&game).unwrap();
    assert_eq!(FourPlayerBot::fallback(&input), DoudizhuAction::Pass);
}

#[test]
fn farmer_before_landlord_guards_a_single_card_trick_with_a_high_card() {
    let mut game = position([&[0, 9, 10], &[4, 6, 8], &[5, 7, 9], &[1, 11, 12]], 0, 0);
    let card = game.hand(PlayerId(0)).unwrap()[0];
    game.act(PlayerId(0), DoudizhuAction::Play(vec![card]))
        .unwrap();
    game.act(PlayerId(1), DoudizhuAction::Pass).unwrap();
    game.act(PlayerId(2), DoudizhuAction::Pass).unwrap();
    let input = FourPlayerObservation::from_game(&game).unwrap();
    let DoudizhuAction::Play(cards) = FourPlayerBot::fallback(&input) else {
        panic!("must guard")
    };
    assert_eq!(cards.len(), 1);
    assert!(cards[0].rank() >= 11);
}

#[test]
fn preserves_a_large_bomb_while_shedding_an_isolated_single() {
    let game = position(
        [&[0, 4, 4, 4, 4, 4, 4, 4, 4], &[1, 7], &[2, 8], &[3, 9]],
        0,
        0,
    );
    let input = FourPlayerObservation::from_game(&game).unwrap();
    let DoudizhuAction::Play(cards) = FourPlayerBot::fallback(&input) else {
        panic!("must lead")
    };
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].rank(), 0);
}

#[test]
fn simulations_publish_completed_sample_sweeps_and_choose_legal_moves() {
    let mut game = position([&[0, 10], &[1, 8], &[2, 7], &[3, 6]], 0, 0);
    let input = FourPlayerObservation::from_game(&game).unwrap();
    let mut updates = 0;
    let action = FourPlayerBot::choose_with_progress(
        &input,
        &SearchBudget::new(Duration::from_secs(2)),
        &mut |action| {
            updates += 1;
            let mut probe = game.clone();
            probe.act(probe.current, action).unwrap();
        },
    );
    assert!(
        updates > 1,
        "must evaluate sampled deals, not just the heuristic"
    );
    game.act(game.current, action).unwrap();
}

#[test]
fn both_four_player_variants_complete_games_with_legal_fallback_moves() {
    for allow_single_wings in [false, true] {
        let rules = DoudizhuRuleSet {
            mode: DoudizhuMode::Four,
            allow_single_wings,
            ..Default::default()
        };
        let mut game =
            DoudizhuGameState::new(rules, build_deck(DoudizhuMode::Four), PlayerId(0)).unwrap();
        game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
        for _ in 0..432 {
            if game.phase == DoudizhuPhase::Finished {
                break;
            }
            let input = FourPlayerObservation::from_game(&game).unwrap();
            game.act(game.current, FourPlayerBot::fallback(&input))
                .unwrap();
        }
        assert_eq!(game.phase, DoudizhuPhase::Finished);
    }
}
