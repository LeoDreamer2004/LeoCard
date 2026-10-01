use super::fixtures::*;
use leocard_texas_holdem::{
    TexasHoldemAction as Action, TexasHoldemHandCategory as Category,
    TexasHoldemPlayerId as Player, TexasHoldemRank as R, TexasHoldemSuit as S,
};

#[test]
fn match_statistics_follow_rotating_hands_and_reset_streaks_on_repetition_and_loss() {
    let hands = [
        [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
        [card(R::King, S::Spade), card(R::King, S::Heart)],
        [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
    ];
    let boards = [
        [
            card(R::Two, S::Diamond),
            card(R::Four, S::Club),
            card(R::Seven, S::Heart),
            card(R::Nine, S::Spade),
            card(R::Jack, S::Diamond),
        ],
        [
            card(R::Ace, S::Diamond),
            card(R::Four, S::Club),
            card(R::Seven, S::Heart),
            card(R::Nine, S::Spade),
            card(R::Jack, S::Diamond),
        ],
        [
            card(R::Two, S::Spade),
            card(R::Four, S::Spade),
            card(R::Seven, S::Spade),
            card(R::Nine, S::Spade),
            card(R::Jack, S::Diamond),
        ],
    ];
    let mut game = game(&hands, boards[0], vec![50; 3]);
    for (index, board) in boards.iter().enumerate() {
        if index > 0 {
            game.start_next_hand(deck(&hands, *board, (game.dealer().0 + 1) % 3))
                .unwrap();
        }
        check_down(&mut game);
        let facts = game.hand_statistics(Player(0)).unwrap();
        assert_eq!(facts.match_statistics.hands, index as u32 + 1);
        assert_eq!(facts.match_statistics.won_hands, index as u32 + 1);
        assert_eq!(
            facts.match_statistics.consecutive_winning_categories.len(),
            index + 1
        );
    }
    game.start_next_hand(deck(&hands, boards[2], (game.dealer().0 + 1) % 3))
        .unwrap();
    check_down(&mut game);
    assert_eq!(
        game.hand_statistics(Player(0))
            .unwrap()
            .match_statistics
            .consecutive_winning_categories,
        vec![Category::Flush]
    );
    game.start_next_hand(deck(&hands, boards[0], (game.dealer().0 + 1) % 3))
        .unwrap();
    post_blinds(&mut game);
    while game.current_player() != Some(Player(0)) {
        let player = game.current_player().unwrap();
        game.act(
            player,
            if game.amount_to_call(player).unwrap() > 0 {
                Action::Call
            } else {
                Action::Check
            },
        )
        .unwrap();
    }
    game.act(Player(0), Action::Fold).unwrap();
    check_down(&mut game);
    let facts = game.hand_statistics(Player(0)).unwrap();
    assert_eq!(facts.match_statistics.consecutive_folds, 1);
    assert!(
        facts
            .match_statistics
            .consecutive_winning_categories
            .is_empty()
    );
    game.start_next_hand(deck(&hands, boards[0], (game.dealer().0 + 1) % 3))
        .unwrap();
    check_down(&mut game);
    assert_eq!(
        game.hand_statistics(Player(0))
            .unwrap()
            .match_statistics
            .consecutive_folds,
        0
    );
}

#[test]
fn a_blind_is_included_in_the_voluntary_all_in_wager() {
    let hands = [
        [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
        [card(R::King, S::Spade), card(R::King, S::Heart)],
        [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
    ];
    let board = [
        card(R::Two, S::Diamond),
        card(R::Four, S::Club),
        card(R::Seven, S::Heart),
        card(R::Nine, S::Spade),
        card(R::Jack, S::Diamond),
    ];
    let mut game = game(&hands, board, vec![40; 3]);
    post_blinds(&mut game);
    game.act(Player(0), Action::Call).unwrap();
    game.act(Player(1), Action::AllIn).unwrap();
    assert_eq!(game.last_action_statistics().unwrap().all_in_amount, 40);
}
