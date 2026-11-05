use super::fixtures::*;
use crate::texas::hand_category_index;
use leocard_texas_holdem::{TexasHandCategory, TexasPhase, evaluate_player_hand};

#[test]
fn profile_counts_the_strongest_board_once_regardless_of_run_order() {
    let profiles = [false, true].map(|swap_boards| {
        let mut session = voting_session_with_deck(winning_deck(swap_boards));
        vote(&mut session, 20, true);
        vote(&mut session, 21, true);
        vote(&mut session, 22, true);
        assert_hand_completion(&session, true);

        let adapter = session.game().unwrap();
        let game = adapter.game();
        let dealer = game.dealer().index();
        let TexasPhase::Complete(result) = game.phase() else {
            panic!("the final vote must settle both boards");
        };
        let hole = game.players()[dealer].hole_cards();
        let first = evaluate_player_hand(hole, game.community(), game.rules()).unwrap();
        let second = evaluate_player_hand(
            hole,
            &result.second_run.as_ref().unwrap().community,
            game.rules(),
        )
        .unwrap();
        let expected = if swap_boards {
            [TexasHandCategory::ThreeOfAKind, TexasHandCategory::OnePair]
        } else {
            [TexasHandCategory::OnePair, TexasHandCategory::ThreeOfAKind]
        };
        assert_eq!([first.category(), second.category()], expected);

        let counts = (0..3)
            .map(|offset| {
                let player = adapter.players()[(dealer + offset) % 3].id;
                session
                    .room
                    .players
                    .iter()
                    .find(|p| p.id == player)
                    .unwrap()
                    .game_profiles
                    .texas
                    .as_ref()
                    .unwrap()
                    .hand_category_counts
            })
            .collect::<Vec<_>>();
        assert_eq!(
            counts[0][hand_category_index(TexasHandCategory::ThreeOfAKind)],
            1
        );
        assert_eq!(
            counts[0][hand_category_index(TexasHandCategory::OnePair)],
            0
        );
        assert!(
            counts
                .iter()
                .all(|categories| categories.iter().sum::<u32>() == 1)
        );
        counts
    });
    assert_eq!(profiles[0], profiles[1]);
}
