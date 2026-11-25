use super::MahjongModelHistory;
use super::features::encode;
use crate::{
    MahjongBotAction, MahjongClaim, MahjongGameState, MahjongPhase, MahjongRuleSet, build_deck,
};
use leocard_game_common::PlayerId;

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
