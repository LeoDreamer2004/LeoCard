use super::model::Model;
use super::{DouzeroBot, DouzeroObservation};
use crate::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, bot_action,
    build_deck,
};
use leocard_game_common::{PlayerId, SearchBudget};
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
