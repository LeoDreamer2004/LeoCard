use super::fixtures::game;
use crate::{ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiMode, ShengjiPhase};
use leocard_game_common::SearchBudget;
use std::time::Duration;

#[test]
fn four_and_six_player_bots_finish_legal_hands_with_cancelled_search() {
    for mode in [ShengjiMode::Classic, ShengjiMode::SixPlayer] {
        let mut game = game(mode, 203);
        let budget = SearchBudget::new(Duration::from_secs(1));
        budget.cancel();
        for _ in 0..200 {
            if matches!(game.phase(), ShengjiPhase::Finished(_)) {
                break;
            }
            let player = game.current_player().unwrap();
            let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
            let ShengjiBotAction::Play(cards) = ShengjiBot::choose(&observation, &budget) else {
                panic!("play")
            };
            game.play_cards(player, &cards).unwrap();
        }
        assert!(matches!(game.phase(), ShengjiPhase::Finished(_)));
    }
}
