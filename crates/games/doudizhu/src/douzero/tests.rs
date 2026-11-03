use super::actions::legal_actions;
use super::model::Model;
use super::{DouzeroBot, DouzeroObservation};
use crate::{
    DoudizhuAction, DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet,
    bot_action, build_deck, classify,
};
use leocard_game_common::{PlayerId, SearchBudget};
use std::collections::BTreeSet;
use std::time::Duration;

#[test]
fn native_inference_matches_pytorch_for_all_three_roles() {
    for (role, bytes, width) in [
        (
            0,
            include_bytes!("../../models/douzero/landlord.fixture").as_slice(),
            373,
        ),
        (
            1,
            include_bytes!("../../models/douzero/landlord_down.fixture").as_slice(),
            484,
        ),
        (
            2,
            include_bytes!("../../models/douzero/landlord_up.fixture").as_slice(),
            484,
        ),
    ] {
        let values: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|part| f32::from_le_bytes(*part))
            .collect();
        let model = Model::for_role(role);
        let hidden = model.encode_history(&values[..810]);
        let rows: Vec<_> = values[810..810 + 32 * width]
            .chunks_exact(width)
            .map(|row| row.to_vec())
            .collect();
        let expected = &values[810 + 32 * width..];
        let actual = model.scores(&hidden, &rows);
        for (actual, expected) in actual.iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 0.001,
                "role {role}: {actual} != {expected}"
            );
        }
    }
}

#[test]
fn legal_actions_cover_every_rank_subset_including_wings() {
    let rules = DoudizhuRuleSet::default();
    for hand in [
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 12, 13, 52, 53],
        vec![0, 1, 2, 4, 5, 6, 8, 12, 16, 20, 24, 28, 32, 36],
    ] {
        let hand: Vec<_> = hand.into_iter().map(DoudizhuCard).collect();
        for table in [None, classify(&[DoudizhuCard(8)], rules)] {
            let actual: BTreeSet<Vec<_>> = legal_actions(&hand, table.as_ref(), rules)
                .into_iter()
                .filter_map(|action| match action {
                    DoudizhuAction::Play(cards) => {
                        Some(cards.iter().map(|card| card.rank()).collect())
                    }
                    _ => None,
                })
                .collect();
            let mut expected = BTreeSet::new();
            for mask in 1..1usize << hand.len() {
                let cards: Vec<_> = hand
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| mask & (1 << index) != 0)
                    .map(|(_, card)| *card)
                    .collect();
                if classify(&cards, rules)
                    .is_some_and(|play| table.as_ref().is_none_or(|last| play.beats(last)))
                {
                    expected.insert(cards.iter().map(|card| card.rank()).collect());
                }
            }
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn observations_ignore_opponents_private_card_distribution() {
    let deck = build_deck(DoudizhuMode::Three);
    let mut swapped = deck.clone();
    swapped.swap(1, 5);
    let mut observations = Vec::new();
    for deck in [deck, swapped] {
        let mut game =
            DoudizhuGameState::new(DoudizhuRuleSet::default(), deck, PlayerId(0)).unwrap();
        game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
        observations.push(DouzeroObservation::from_game(&game, &[]).unwrap());
    }
    assert_eq!(observations[0], observations[1]);
}

#[test]
fn neural_and_existing_policies_complete_a_legal_game() {
    let mut game = DoudizhuGameState::new(
        DoudizhuRuleSet::default(),
        build_deck(DoudizhuMode::Three),
        PlayerId(0),
    )
    .unwrap();
    game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
    let mut history = Vec::new();
    for _ in 0..300 {
        if game.phase == DoudizhuPhase::Finished {
            return;
        }
        let action = if history.len() % 3 == 0 {
            let input = DouzeroObservation::from_game(&game, &history).unwrap();
            DouzeroBot::choose_with_progress(
                &input,
                &SearchBudget::new(Duration::from_secs(10)),
                &mut |_| {},
            )
        } else {
            bot_action(&game).unwrap()
        };
        let player = game.current;
        game.act(player, action.clone()).unwrap();
        history.push((player, action));
    }
    panic!("game did not finish");
}
