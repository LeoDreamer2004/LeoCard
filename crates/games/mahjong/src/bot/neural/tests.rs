use super::MahjongModelHistory;
use super::features::encode;
use crate::{
    MahjongBotAction, MahjongClaim, MahjongGameState, MahjongPhase, MahjongRuleSet, build_deck,
};
use leocard_ai::{DevicePreference, ModelId, model_spec};
use leocard_game_common::PlayerId;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "requires downloaded checkpoints and PyTorch fixtures in LEOCARD_AI_TEST_MODELS"]
fn native_inference_matches_pytorch() {
    let root = PathBuf::from(env::var_os("LEOCARD_AI_TEST_MODELS").expect("model directory"));
    let spec = model_spec(ModelId::Kdens3);
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
        assert_eq!(fixture.len(), 1368 + 235);
        let device = if env::var_os("LEOCARD_AI_TEST_GPU").is_some() {
            DevicePreference::Gpu
        } else {
            DevicePreference::Cpu
        };
        let network = super::model::build(&tensors, device).unwrap();
        let actual = network.run(&fixture[..1368], 1).unwrap();
        let expected = &fixture[1368..];
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
fn claims_keep_published_feature_cache_and_current_legal_actions() {
    let mut deck = build_deck();
    deck.swap(1, 4);
    deck.swap(2, 5);
    let discarded = deck[0];
    let mut game =
        MahjongGameState::new_with_deck(MahjongRuleSet::default(), deck, PlayerId(0)).unwrap();
    while matches!(game.phase(), MahjongPhase::Dealing { .. }) {
        game.advance_deal().unwrap();
    }
    let mut history = MahjongModelHistory::default();
    history.refresh(&game);
    game.discard(PlayerId(0), discarded).unwrap();
    let response = history
        .observation(&game, PlayerId(1))
        .expect("a real claim opportunity");
    assert!(
        response
            .position
            .options
            .contains(&MahjongBotAction::Respond(MahjongClaim::Pung))
    );
    // The current discard belongs in the legal mask, but not the source agent's
    // cached features until that player changes their hand.
    assert_ne!(response.features.as_ref(), encode(&response.position));
    while let MahjongPhase::WaitingForClaims(pending) = game.phase() {
        let player = pending.waiting_for()[0];
        let action = if player == PlayerId(1) {
            MahjongClaim::Pung
        } else {
            MahjongClaim::Pass
        };
        game.respond_to_claim(player, action).unwrap();
    }
    let after = history.observation(&game, PlayerId(1)).unwrap();
    assert_eq!(after.features.as_ref(), encode(&after.position));
    assert_ne!(after.features, response.features);
}
