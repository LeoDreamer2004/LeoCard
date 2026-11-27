use super::policy::build;
use super::{TexasPpoBot, TexasPpoMemory, TexasPpoObservation};
use crate::{TexasAction, TexasCard, TexasGameState, TexasRuleSet, build_deck};
use leocard_ai::{DevicePreference, ModelId, model_spec};
use leocard_game_common::PlayerId;
use std::path::PathBuf;
use std::{env, fs};

fn game(deck: Vec<TexasCard>) -> TexasGameState {
    let mut game = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 6,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        deck,
    )
    .unwrap();
    while game.blind_to_post().is_some() {
        game.act(game.current_player().unwrap(), TexasAction::PostBlind)
            .unwrap();
    }
    game
}

#[test]
fn six_player_features_hide_opponent_cards_and_only_offer_legal_actions() {
    let first = game(build_deck(false));
    let player = first.current_player().unwrap();
    let ours = first.players()[player.index()].hole_cards();
    let mut deck = build_deck(false);
    let hidden: Vec<_> = deck
        .iter()
        .enumerate()
        .filter(|(_, card)| !ours.contains(card))
        .map(|(i, _)| i)
        .take(2)
        .collect();
    deck.swap(hidden[0], hidden[1]);
    let second = game(deck);
    let memory = TexasPpoMemory::default();
    let input = TexasPpoObservation::from_game(&first, player, &memory).unwrap();
    assert_eq!(
        Some(&input),
        TexasPpoObservation::from_game(&second, player, &memory).as_ref()
    );
    assert_eq!(
        input.features[52],
        first.players()[player.index()].committed_total() as f32
    );
    assert_eq!(input.features[53], 2.0);
    for (_, action) in input.actions {
        first.clone().act(player, action).unwrap();
    }
    assert!(!TexasPpoBot::supports_rules(&TexasRuleSet {
        short_deck: true,
        ..TexasRuleSet::default()
    }));
    assert!(!TexasPpoBot::supports_rules(&TexasRuleSet {
        side_pots: false,
        ..TexasRuleSet::default()
    }));
}

fn reference(device: DevicePreference) {
    let root =
        PathBuf::from(env::var_os("LEOCARD_AI_TEST_MODELS").expect("external model directory"))
            .join("texas-ppo");
    let (_, tensors) = model_spec(ModelId::TexasPpo).files[0]
        .convert(&fs::read(root.join("policy.source")).unwrap())
        .unwrap();
    let network = build(&tensors, device).unwrap();
    let bytes = fs::read(root.join("policy.fixture")).unwrap();
    let values: Vec<_> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| f32::from_le_bytes(*v))
        .collect();
    let actual = network.run(&values[..7 * 84], 7).unwrap();
    for (actual, expected) in actual.iter().zip(&values[7 * 84..]) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }
}

#[test]
#[ignore = "requires externally downloaded models and PyTorch references"]
fn ppo_matches_pytorch_on_cpu() {
    reference(DevicePreference::Cpu);
}

#[test]
#[ignore = "requires externally downloaded models, references and a hardware GPU"]
fn ppo_matches_pytorch_on_gpu() {
    reference(DevicePreference::Gpu);
}
