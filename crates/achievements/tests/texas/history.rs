use super::fixtures::*;
use leocard_achievements::AchievementBook;
use leocard_texas_holdem::{
    Phase, TexasHoldemAction as Action, TexasHoldemCard, TexasHoldemHandCategory as Category,
    TexasHoldemPlayerId as Player, TexasHoldemRank as R, TexasHoldemStreet as Street,
    TexasHoldemSuit as S,
};

fn board() -> [TexasHoldemCard; 5] {
    [
        card(R::Two, S::Diamond),
        card(R::Four, S::Club),
        card(R::Seven, S::Heart),
        card(R::Nine, S::Spade),
        card(R::Jack, S::Diamond),
    ]
}

#[test]
fn full_three_bet_survives_street_transition_and_invalid_actions_leave_no_trace() {
    let mut game = game(
        &[
            [card(R::King, S::Spade), card(R::King, S::Heart)],
            [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
            [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
        ],
        board(),
        vec![50; 3],
    );
    post_blinds(&mut game);
    game.act(Player(0), Action::RaiseTo(4)).unwrap();
    let before = game.clone();
    assert!(game.act(Player(1), Action::RaiseTo(5)).is_err());
    assert_eq!(before, game);
    game.act(Player(1), Action::RaiseTo(6)).unwrap();
    let statistics = game.last_action_statistics().unwrap();
    assert!(statistics.full_raise);
    assert_eq!(statistics.bet_level, 3);
    game.act(Player(2), Action::Fold).unwrap();
    check_down(&mut game);
    let facts = game.hand_statistics(Player(1)).unwrap();
    assert_eq!(facts.preflop_bet_levels, vec![3]);
    assert!(facts.won_chips > 0);
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    assert!(achieved(&book, "premium_three_bet"));
    assert!(achieved(&book, "aces_vs_pair"));
}

#[test]
fn short_all_in_does_not_make_a_three_bet_and_refund_is_not_a_win() {
    let hands = [
        [card(R::King, S::Spade), card(R::King, S::Heart)],
        [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
        [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
    ];
    let mut short = game(&hands, board(), vec![20, 5, 20]);
    post_blinds(&mut short);
    short.act(Player(0), Action::RaiseTo(4)).unwrap();
    short.act(Player(1), Action::AllIn).unwrap();
    let statistics = short.last_action_statistics().unwrap();
    assert!(!statistics.full_raise);
    assert_eq!(statistics.bet_level, 2);
    let mut refund = game(&hands, board(), vec![20, 5, 5]);
    post_blinds(&mut refund);
    refund.act(Player(0), Action::AllIn).unwrap();
    check_down(&mut refund);
    let facts = refund.hand_statistics(Player(0)).unwrap();
    assert_eq!(facts.final_stack, 15);
    assert_eq!(facts.won_chips, 0);
    assert_eq!(facts.match_statistics.won_hands, 0);
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    assert!(!achieved(&book, "first_pot"));
}

#[test]
fn side_pot_raise_is_recorded_only_with_two_live_side_pot_players() {
    let mut game = game(
        &[
            [card(R::King, S::Spade), card(R::King, S::Heart)],
            [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
            [card(R::Jack, S::Spade), card(R::Jack, S::Heart)],
            [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
        ],
        board(),
        vec![100, 5, 8, 100],
    );
    post_blinds(&mut game);
    for (player, action) in [
        (3, Action::Call),
        (0, Action::Call),
        (1, Action::AllIn),
        (2, Action::AllIn),
        (3, Action::RaiseTo(14)),
        (0, Action::Call),
    ] {
        game.act(Player(player), action).unwrap();
    }
    check_down(&mut game);
    let facts = game.hand_statistics(Player(3)).unwrap();
    assert_eq!(facts.side_pot_raises, 1);
    assert_eq!(facts.won_chips, 12);
    assert_eq!(facts.won_side_pot_chips, 12);
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    assert!(achieved(&book, "side_pot_raise"));
}

#[test]
fn all_in_pairs_and_quads_defeat_come_from_actual_shared_pots() {
    let mut pairs = game(
        &[
            [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
            [card(R::King, S::Spade), card(R::King, S::Heart)],
            [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
        ],
        board(),
        vec![20; 3],
    );
    post_blinds(&mut pairs);
    pairs.act(Player(0), Action::AllIn).unwrap();
    check_down(&mut pairs);
    let facts = pairs.hand_statistics(Player(0)).unwrap();
    assert_eq!(facts.defeated_preflop_all_in_pairs, 2);
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    assert!(achieved(&book, "three_pocket_pairs"));
    assert!(achieved(&book, "all_chips"));
    let mut duel = game(
        &[
            [card(R::Nine, S::Spade), card(R::Eight, S::Spade)],
            [card(R::Five, S::Club), card(R::Ace, S::Heart)],
            [card(R::King, S::Club), card(R::Queen, S::Diamond)],
        ],
        [
            card(R::Seven, S::Spade),
            card(R::Six, S::Spade),
            card(R::Five, S::Spade),
            card(R::Five, S::Heart),
            card(R::Five, S::Diamond),
        ],
        vec![20; 3],
    );
    post_blinds(&mut duel);
    duel.act(Player(0), Action::AllIn).unwrap();
    check_down(&mut duel);
    let facts = duel.hand_statistics(Player(0)).unwrap();
    assert_eq!(facts.category, Some(Category::StraightFlush));
    assert!(facts.defeated_categories.contains(&Category::FourOfAKind));
    trigger(&mut book, facts, 2);
    assert!(achieved(&book, "straight_flush_vs_quads"));
}

#[test]
fn river_value_and_bluff_are_derived_without_revealing_opponent_cards() {
    let mut value = game(
        &[
            [card(R::King, S::Spade), card(R::King, S::Heart)],
            [card(R::Ace, S::Spade), card(R::Ace, S::Heart)],
            [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
        ],
        board(),
        vec![50; 3],
    );
    post_blinds(&mut value);
    while !matches!(value.phase(), Phase::Betting(Street::River)) {
        let player = value.current_player().unwrap();
        value
            .act(
                player,
                if value.amount_to_call(player).unwrap() > 0 {
                    Action::Call
                } else {
                    Action::Check
                },
            )
            .unwrap();
    }
    value.act(Player(1), Action::RaiseTo(4)).unwrap();
    check_down(&mut value);
    assert!(value.hand_statistics(Player(1)).unwrap().river_bet_called);
    let mut bluff = game(
        &[
            [card(R::Queen, S::Spade), card(R::Queen, S::Heart)],
            [card(R::Three, S::Spade), card(R::Nine, S::Heart)],
            [card(R::Ace, S::Spade), card(R::Jack, S::Heart)],
        ],
        [
            card(R::Two, S::Diamond),
            card(R::Five, S::Club),
            card(R::Eight, S::Heart),
            card(R::Queen, S::Diamond),
            card(R::King, S::Spade),
        ],
        vec![50; 3],
    );
    post_blinds(&mut bluff);
    while !matches!(bluff.phase(), Phase::Betting(Street::River)) {
        let player = bluff.current_player().unwrap();
        bluff
            .act(
                player,
                if bluff.amount_to_call(player).unwrap() > 0 {
                    Action::Call
                } else {
                    Action::Check
                },
            )
            .unwrap();
    }
    bluff.act(Player(1), Action::AllIn).unwrap();
    bluff.act(Player(2), Action::Fold).unwrap();
    bluff.act(Player(0), Action::Fold).unwrap();
    let facts = bluff.hand_statistics(Player(1)).unwrap();
    assert!(!facts.showdown);
    assert_eq!(facts.category, Some(Category::HighCard));
    assert!(
        facts
            .river_all_in_folded_categories
            .contains(&Category::ThreeOfAKind)
    );
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    assert!(achieved(&book, "river_bluff"));
}
