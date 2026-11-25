use super::{GuandanModelHistory, GuandanModelObservation};
use crate::{GuandanGameState, GuandanRuleSet, build_deck};
use leocard_ai::{DevicePreference, ModelId, model_spec};
use leocard_game_common::PlayerId;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "requires downloaded checkpoints and PyTorch fixtures in LEOCARD_AI_TEST_MODELS"]
fn native_inference_matches_pytorch() {
    let root = PathBuf::from(env::var_os("LEOCARD_AI_TEST_MODELS").expect("model directory"));
    let spec = model_spec(ModelId::GuandanActor);
    for file in spec.files {
        let path = root.join(spec.key).join(file.name);
        let (_, tensors) = file
            .convert(&fs::read(path.with_extension("source")).unwrap())
            .unwrap();
        let bytes = fs::read(path.with_extension("fixture")).unwrap();
        let fixture: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|n| f32::from_le_bytes(*n))
            .collect();
        assert_eq!(fixture.len(), 3049 + 375);
        let device = if env::var_os("LEOCARD_AI_TEST_GPU").is_some() {
            DevicePreference::Gpu
        } else {
            DevicePreference::Cpu
        };
        let network = super::policy::build(&tensors, device).unwrap();
        let actual = network.run(&fixture[..3049], 1).unwrap();
        let expected = &fixture[3049..];
        let max_error = actual
            .iter()
            .zip(expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        eprintln!("{} {device:?} max error {max_error}", file.name);
        for (a, b) in actual.iter().zip(expected) {
            assert!((a - b).abs() < 0.002 * (1.0 + b.abs()), "{a} != {b}");
        }
    }
}

#[test]
fn model_observation_cannot_see_opponents_cards() {
    let mut deck = build_deck();
    let first =
        GuandanGameState::new(GuandanRuleSet::default(), deck.clone(), PlayerId(0)).unwrap();
    deck.swap(1, 2);
    deck.swap(5, 7);
    let second = GuandanGameState::new(GuandanRuleSet::default(), deck, PlayerId(0)).unwrap();
    let history = GuandanModelHistory::default();
    assert_ne!(
        first.player(PlayerId(1)).unwrap().hand(),
        second.player(PlayerId(1)).unwrap().hand()
    );
    let observation = GuandanModelObservation::from_game(&first, &history).unwrap();
    assert_eq!(
        Some(&observation),
        GuandanModelObservation::from_game(&second, &history).as_ref()
    );
    for (_, action) in observation.actions {
        first.clone().act(PlayerId(0), action).unwrap();
    }
}
