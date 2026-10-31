use BlackjackAction::{Bet, DoubleDown, Split, Stand};
use BlackjackRank::{Ace, Eight, Four, King, Nine, Six, Ten, Three, Two};
use leocard_blackjack::{
    BlackjackAction, BlackjackCard, BlackjackGameError, BlackjackGameState, BlackjackHandResult,
    BlackjackHandValue, BlackjackPhase, BlackjackRank, BlackjackRuleSet, BlackjackSuit, build_shoe,
};
use leocard_game_common::PlayerId;

fn shoe(prefix: &[BlackjackRank]) -> Vec<BlackjackCard> {
    let mut remaining = build_shoe();
    let mut ordered = Vec::new();
    for &rank in prefix {
        let card = BlackjackCard {
            suit: BlackjackSuit::Spade,
            rank,
        };
        let index = remaining
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        ordered.push(remaining.remove(index));
    }
    ordered.extend(remaining);
    ordered
}

fn game(prefix: &[BlackjackRank]) -> BlackjackGameState {
    let mut game = BlackjackGameState::new(BlackjackRuleSet::default(), 1, shoe(prefix)).unwrap();
    game.act(PlayerId(0), Bet { amount: 2 }).unwrap();
    game
}

#[test]
fn aces_change_value_and_dealer_stands_on_soft_seventeen() {
    let cards = [Ace, Ace, Nine].map(|rank| BlackjackCard {
        suit: BlackjackSuit::Club,
        rank,
    });
    assert_eq!(
        BlackjackHandValue::of(&cards),
        BlackjackHandValue {
            total: 21,
            soft: true
        }
    );
    let mut game = game(&[Ten, Ace, Eight, Six, King]);
    assert_eq!(game.visible_dealer().len(), 1);
    assert!(game.dealer_has_hidden_card());
    game.act(PlayerId(0), Stand).unwrap();
    assert_eq!(game.visible_dealer().len(), 2);
    assert_eq!(game.players()[0].stack, 202);
}

#[test]
fn natural_pays_three_to_two_but_pushes_against_dealer_natural() {
    for (prefix, stack, result) in [
        ([Ace, Ten, King, Six], 203, BlackjackHandResult::Blackjack),
        ([Ace, Ace, King, King], 200, BlackjackHandResult::Push),
        ([Nine, Ace, Ten, King], 198, BlackjackHandResult::Loss),
    ] {
        let game = game(&prefix);
        assert_eq!(game.phase(), BlackjackPhase::Complete);
        assert_eq!(game.players()[0].stack, stack);
        assert_eq!(game.players()[0].hands[0].result, Some(result));
    }
}

#[test]
fn split_aces_receive_one_card_and_do_not_become_naturals() {
    let mut game = game(&[Ace, Six, Ace, Ten, Ten, Nine, BlackjackRank::Five]);
    game.act(PlayerId(0), Split).unwrap();
    assert_eq!(game.phase(), BlackjackPhase::Complete);
    assert_eq!(game.players()[0].stack, 198);
    assert_eq!(
        game.players()[0].hands[0].result,
        Some(BlackjackHandResult::Push)
    );
    assert_eq!(game.players()[0].statistics.blackjacks, 0);
}

#[test]
fn double_after_split_settles_each_hand_once_and_preserves_round_stack() {
    let mut game = game(&[Eight, Six, Eight, Ten, Three, Two, Ten, Ten, Four]);
    game.act(PlayerId(0), Split).unwrap();
    game.act(PlayerId(0), DoubleDown).unwrap();
    assert_eq!(game.current_hand(), Some(1));
    game.act(PlayerId(0), DoubleDown).unwrap();
    let player = &game.players()[0];
    assert_eq!(player.stack, 204);
    assert_eq!(player.round_start_stack, 200);
    assert_eq!(player.statistics.hands, 2);
    assert_eq!(player.statistics.doubles, 2);
    assert_eq!(player.statistics.splits, 1);
    assert_eq!(
        player.hands.iter().map(|hand| hand.net_chips).sum::<i32>(),
        4
    );
    assert_eq!(
        game.act(PlayerId(0), Stand),
        Err(BlackjackGameError::NotPlayersTurn)
    );
    game.start_next_round(shoe(&[])).unwrap();
    assert_eq!(game.players()[0].stack, 204);
    assert_eq!(game.players()[0].round_start_stack, 204);
    assert_eq!(game.phase(), BlackjackPhase::Betting);
}

#[test]
fn invalid_bets_and_shoes_leave_no_partially_applied_action() {
    let mut game = BlackjackGameState::new(BlackjackRuleSet::default(), 2, shoe(&[])).unwrap();
    assert_eq!(
        game.act(PlayerId(1), Bet { amount: 2 }),
        Err(BlackjackGameError::NotPlayersTurn)
    );
    assert_eq!(
        game.act(PlayerId(0), Bet { amount: 3 }),
        Err(BlackjackGameError::InvalidBet)
    );
    assert_eq!(game.players()[0].stack, 200);
    assert!(game.players()[0].hands.is_empty());
    let invalid = vec![
        BlackjackCard {
            suit: BlackjackSuit::Club,
            rank: Ace
        };
        312
    ];
    assert!(matches!(
        BlackjackGameState::new(BlackjackRuleSet::default(), 2, invalid),
        Err(BlackjackGameError::InvalidShoe)
    ));
}
