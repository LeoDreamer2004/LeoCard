use crate::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, FourPlayerBot,
    FourPlayerObservation, build_deck,
};
use leocard_game_common::PlayerId;

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
