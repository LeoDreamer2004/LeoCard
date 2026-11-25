use super::model::{Model, build_head};
use super::{DouzeroBot, DouzeroObservation};
use crate::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, bot_action,
    build_deck,
};
use leocard_ai::{DevicePreference, ModelId, ModelRegistry, TensorMap, model_spec};
use leocard_game_common::{PlayerId, SearchBudget};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[test]
#[ignore = "requires external checkpoints and PyTorch references in LEOCARD_AI_TEST_MODELS"]
fn native_inference_matches_pytorch_for_all_three_roles() {
    let tensors = downloaded_tensors();
    let spec = model_spec(ModelId::Douzero);
    let directory = PathBuf::from(env::var_os("LEOCARD_AI_TEST_MODELS").expect("model directory"))
        .join(spec.key);
    for (role, file) in spec.files.iter().enumerate() {
        let width = if role == 0 { 373 } else { 484 };
        let bytes = fs::read(directory.join(file.name).with_extension("fixture")).unwrap();
        let values: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|part| f32::from_le_bytes(*part))
            .collect();
        let model = Model::read(&tensors[role]).unwrap();
        let hidden = model.encode_history(&values[..810]);
        let rows: Vec<_> = values[810..810 + 32 * width]
            .chunks_exact(width)
            .map(|row| row.to_vec())
            .collect();
        let expected = &values[810 + 32 * width..];
        let batch: Vec<_> = rows
            .iter()
            .flat_map(|row| hidden.iter().chain(row).copied())
            .collect();
        let devices: &[DevicePreference] = if env::var_os("LEOCARD_AI_TEST_GPU").is_some() {
            &[DevicePreference::Cpu, DevicePreference::Gpu]
        } else {
            &[DevicePreference::Cpu]
        };
        for &device in devices {
            let actual = build_head(&tensors[role], width, device)
                .unwrap()
                .run(&batch, rows.len())
                .unwrap();
            for (actual, expected) in actual.iter().zip(expected) {
                assert!(
                    (actual - expected).abs() < 0.001,
                    "role {role}: {actual} != {expected}"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires downloaded DouZero checkpoints in LEOCARD_AI_TEST_MODELS"]
fn neural_and_existing_policies_complete_a_legal_game() {
    ModelRegistry::install(ModelId::Douzero, downloaded_tensors()).unwrap();
    let model = ModelRegistry::get(ModelId::Douzero, DevicePreference::Cpu).unwrap();
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
                &model,
                &SearchBudget::new(Duration::from_secs(10)),
                &mut |_| {},
            )
            .unwrap()
        } else {
            bot_action(&game).unwrap()
        };
        let player = game.current;
        game.act(player, action.clone()).unwrap();
        history.push((player, action));
    }
    panic!("game did not finish");
}

fn downloaded_tensors() -> Vec<TensorMap> {
    let directory =
        PathBuf::from(env::var_os("LEOCARD_AI_TEST_MODELS").expect("set LEOCARD_AI_TEST_MODELS"));
    model_spec(ModelId::Douzero)
        .files
        .iter()
        .map(|file| {
            let bytes = fs::read(
                directory
                    .join(model_spec(ModelId::Douzero).key)
                    .join(format!("{}.source", file.name)),
            )
            .unwrap();
            file.convert(&bytes).unwrap().1
        })
        .collect()
}
