use super::fixtures::*;
use leocard_achievements::{
    AchievementBook, AchievementCategory, AchievementContext, AchievementTrigger, achievements_in,
};
use leocard_protocol::{GameEvent, MatchId, PlayerId, TexasHoldemEvent};
use leocard_texas_holdem::{
    TexasHoldemAction, TexasHoldemActionStatistics, TexasHoldemHandCategory as Category,
    TexasHoldemHandStatistics, TexasHoldemMatchStatistics, TexasHoldemRank as Rank,
    TexasHoldemStreet, TexasHoldemSuit as Suit,
};

#[test]
fn catalogue_has_every_designed_texas_achievement_and_wins_count_once() {
    assert_eq!(
        achievements_in(AchievementCategory::TexasHoldem).count(),
        36
    );
    let mut book = AchievementBook::default();
    for sequence in 1..=5000 {
        trigger(
            &mut book,
            TexasHoldemHandStatistics {
                won_chips: 20,
                ..Default::default()
            },
            sequence,
        );
        if sequence == 9 {
            assert!(!achieved(&book, "ten_wins"));
        }
        if sequence == 499 {
            assert!(!achieved(&book, "five_hundred_wins"));
        }
        if sequence == 4999 {
            assert!(!achieved(&book, "five_thousand_wins"));
        }
    }
    for id in [
        "first_pot",
        "ten_wins",
        "five_hundred_wins",
        "five_thousand_wins",
    ] {
        assert!(achieved(&book, id));
    }
    let before = book.counts();
    trigger(
        &mut book,
        TexasHoldemHandStatistics {
            won_chips: 20,
            ..Default::default()
        },
        5000,
    );
    assert_eq!(before, book.counts());
}

#[test]
fn hand_categories_require_a_won_pot_and_keep_royal_flush_as_a_straight_flush() {
    for (category, id) in [
        (Category::HighCard, "high_card"),
        (Category::OnePair, "one_pair"),
        (Category::TwoPair, "two_pair"),
        (Category::ThreeOfAKind, "three_of_a_kind"),
        (Category::Straight, "straight"),
        (Category::Flush, "flush"),
        (Category::FullHouse, "full_house"),
        (Category::FourOfAKind, "four_of_a_kind"),
        (Category::StraightFlush, "straight_flush"),
        (Category::RoyalFlush, "royal_flush"),
    ] {
        let mut book = AchievementBook::default();
        trigger(
            &mut book,
            TexasHoldemHandStatistics {
                category: Some(category),
                ..Default::default()
            },
            1,
        );
        assert!(!achieved(&book, id));
        trigger(
            &mut book,
            TexasHoldemHandStatistics {
                category: Some(category),
                won_chips: 10,
                showdown: true,
                ..Default::default()
            },
            2,
        );
        assert!(achieved(&book, id));
        if category == Category::RoyalFlush {
            assert!(achieved(&book, "straight_flush"));
        }
    }
}

#[test]
fn action_achievements_use_actual_payment_full_raise_and_local_actor() {
    let mut book = AchievementBook::default();
    for (sequence, actor, amount, full_raise, bet_level) in [
        (1, 1, 40, true, 3),
        (2, 0, 39, false, 2),
        (3, 0, 40, true, 3),
    ] {
        book.trigger(
            &AchievementTrigger::Game {
                player: PlayerId(0),
                event: GameEvent::TexasHoldem(TexasHoldemEvent::ActionApplied {
                    player: PlayerId(actor),
                    action: TexasHoldemAction::AllIn,
                    amount,
                    statistics: TexasHoldemActionStatistics {
                        street: TexasHoldemStreet::PreFlop,
                        all_in_amount: amount,
                        full_raise,
                        raised: full_raise,
                        bet_level,
                    },
                }),
            },
            Some(AchievementContext {
                match_id: MatchId([1; 16]),
                hand_index: Some(0),
                sequence,
            }),
            100,
        );
        assert_eq!(achieved(&book, "large_all_in"), sequence == 3);
        assert_eq!(achieved(&book, "three_bet"), sequence == 3);
    }
}

#[test]
fn match_achievements_require_final_settlement_and_their_full_thresholds() {
    let mut facts = TexasHoldemHandStatistics {
        final_stack: 90,
        total_chips: 120,
        first_place: true,
        match_statistics: TexasHoldemMatchStatistics {
            hands: 8,
            won_hands: 8,
            raised_hands: 4,
            lowest_starting_stack: 9,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut book = AchievementBook::default();
    trigger(&mut book, facts.clone(), 1);
    for id in [
        "frequent_raiser",
        "half_chips",
        "three_quarter_chips",
        "short_stack_comeback",
        "perfect_match",
    ] {
        assert!(!achieved(&book, id));
    }
    facts.match_complete = true;
    trigger(&mut book, facts.clone(), 2);
    for id in [
        "frequent_raiser",
        "half_chips",
        "three_quarter_chips",
        "short_stack_comeback",
        "perfect_match",
    ] {
        assert!(achieved(&book, id));
    }
    assert!(!achieved(&book, "all_chips"));
    facts.final_stack = 120;
    trigger(&mut book, facts, 3);
    assert!(achieved(&book, "all_chips"));
    let mut negative = AchievementBook::default();
    trigger(
        &mut negative,
        TexasHoldemHandStatistics {
            match_complete: true,
            final_stack: 59,
            total_chips: 120,
            first_place: false,
            match_statistics: TexasHoldemMatchStatistics {
                hands: 8,
                won_hands: 7,
                raised_hands: 3,
                lowest_starting_stack: 10,
                ..Default::default()
            },
            ..Default::default()
        },
        1,
    );
    for id in [
        "frequent_raiser",
        "half_chips",
        "three_quarter_chips",
        "short_stack_comeback",
        "perfect_match",
        "all_chips",
    ] {
        assert!(!achieved(&negative, id));
    }
}

#[test]
fn positional_premium_and_streak_achievements_use_recipient_facts() {
    let facts = TexasHoldemHandStatistics {
        won_chips: 15,
        button: true,
        big_blind: true,
        defended_blind: true,
        hole_cards: vec![card(Rank::Ace, Suit::Spade), card(Rank::Ace, Suit::Heart)],
        preflop_bet_levels: vec![3, 5],
        defeated_pocket_pairs: 2,
        defeated_preflop_all_in_pairs: 2,
        preflop_all_in: true,
        river_bet_called: true,
        showdown: true,
        match_statistics: TexasHoldemMatchStatistics {
            consecutive_folds: 10,
            consecutive_winning_categories: vec![
                Category::OnePair,
                Category::Flush,
                Category::FullHouse,
            ],
            ..Default::default()
        },
        ..Default::default()
    };
    let mut book = AchievementBook::default();
    trigger(&mut book, facts, 1);
    for id in [
        "button_win",
        "blind_defense",
        "premium_three_bet",
        "aces_vs_pair",
        "three_pocket_pairs",
        "river_value",
        "ten_folds",
        "three_different_wins",
    ] {
        assert!(achieved(&book, id));
    }
    assert!(!achieved(&book, "uncontested"));
}

#[test]
fn two_seven_side_pot_and_bluff_conditions_reject_near_misses() {
    let mut facts = TexasHoldemHandStatistics {
        won_chips: 30,
        won_pot: 39,
        under_the_gun: true,
        hole_cards: vec![card(Rank::Seven, Suit::Heart), card(Rank::Two, Suit::Spade)],
        side_pot_raises: 1,
        ..Default::default()
    };
    let mut book = AchievementBook::default();
    trigger(&mut book, facts.clone(), 1);
    assert!(achieved(&book, "offsuit_two_seven"));
    assert!(!achieved(&book, "utg_two_seven"));
    assert!(!achieved(&book, "side_pot_raise"));
    facts.won_pot = 40;
    facts.won_side_pot_chips = 10;
    trigger(&mut book, facts.clone(), 2);
    assert!(achieved(&book, "utg_two_seven"));
    assert!(achieved(&book, "side_pot_raise"));
    let mut suited = AchievementBook::default();
    facts.hole_cards[0] = card(Rank::Seven, Suit::Spade);
    trigger(&mut suited, facts, 1);
    assert!(!achieved(&suited, "offsuit_two_seven"));
    let mut bluff = AchievementBook::default();
    let mut facts = TexasHoldemHandStatistics {
        won_chips: 40,
        category: Some(Category::HighCard),
        river_all_in_folded_categories: vec![Category::TwoPair],
        ..Default::default()
    };
    trigger(&mut bluff, facts.clone(), 1);
    assert!(!achieved(&bluff, "river_bluff"));
    facts.river_all_in_folded_categories = vec![Category::ThreeOfAKind];
    trigger(&mut bluff, facts, 2);
    assert!(achieved(&bluff, "river_bluff"));
    let mut duel = AchievementBook::default();
    trigger(
        &mut duel,
        TexasHoldemHandStatistics {
            won_chips: 40,
            category: Some(Category::StraightFlush),
            defeated_categories: vec![Category::FourOfAKind],
            ..Default::default()
        },
        1,
    );
    assert!(achieved(&duel, "straight_flush_vs_quads"));
}
