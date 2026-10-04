use super::support::*;
use leocard_shengji::ShengjiBurialStatistics;

#[test]
fn score_achievements_follow_teams_and_raw_promotion_steps() {
    for player in 0..4 {
        let mut facts = statistics(player);
        for steps in [0, 2, 3, 12, 13] {
            facts.result.promoted_team = ShengjiTeamId(1);
            facts.result.promoted_steps = steps;
            assert_eq!(
                hand_amount("attack_win", &facts),
                u64::from(player % 2 == 1)
            );
            assert_eq!(
                hand_amount("attack_three", &facts),
                u64::from(player % 2 == 1 && steps >= 3)
            );
            assert_eq!(
                hand_amount("attack_thirteen", &facts),
                u64::from(player % 2 == 1 && steps >= 13)
            );
        }
        facts.result.collecting_score = 0;
        facts.result.promoted_team = ShengjiTeamId(0);
        assert_eq!(hand_amount("shutout", &facts), u64::from(player % 2 == 0));
        facts.result.collecting_score = 5;
        assert_eq!(hand_amount("shutout", &facts), 0);
        facts.result.levels = [Rank::Ace, Rank::King];
        assert_eq!(hand_amount("ace", &facts), u64::from(player % 2 == 0));
    }
}

#[test]
fn burial_achievements_belong_to_the_actual_dealer() {
    for player in 0..4 {
        let mut facts = statistics(player);
        facts.burial = Some(ShengjiBurialStatistics {
            card_count: 8,
            scoring_card_count: 0,
            points: 0,
        });
        assert_eq!(hand_amount("clean_bottom", &facts), u64::from(player == 0));
        facts.burial.as_mut().unwrap().points = 5;
        assert_eq!(hand_amount("clean_bottom", &facts), 0);
        facts.burial = Some(ShengjiBurialStatistics {
            card_count: 8,
            scoring_card_count: 8,
            points: 80,
        });
        assert_eq!(
            hand_amount("scoring_bottom_defended", &facts),
            u64::from(player == 0)
        );
        facts.bottom_burier = Some(ShengjiPlayerId(1));
        assert_eq!(hand_amount("scoring_bottom_defended", &facts), 0);
        facts.bottom_burier = Some(ShengjiPlayerId(0));
        facts.result.kitty_multiplier = 2;
        assert_eq!(hand_amount("scoring_bottom_defended", &facts), 0);
    }
}

#[test]
fn opening_hands_and_private_ownership_are_required() {
    let mut facts = statistics(1);
    facts.deck_count = 4;
    facts.opening_hand = ShengjiOpeningHandStatistics {
        card_count: 52,
        trump_count: 52,
        joker_count: 8,
    };
    facts.first_trick_cut_dealer = true;
    for id in ["eight_jokers", "all_trump", "first_cut"] {
        assert_eq!(hand_amount(id, &facts), 1);
    }
    let mut trigger = analyzed(facts.clone());
    let AchievementTrigger::Game { player, .. } = &mut trigger else {
        unreachable!()
    };
    *player = PlayerId(3);
    for id in ["eight_jokers", "all_trump", "first_cut", "ten_thousand"] {
        assert_eq!(amount(id, &trigger), 0);
    }
    facts.deck_count = 3;
    assert_eq!(hand_amount("eight_jokers", &facts), 0);
    facts.deck_count = 4;
    facts.opening_hand.joker_count = 7;
    assert_eq!(hand_amount("eight_jokers", &facts), 0);
    facts.opening_hand.trump_count = 51;
    assert_eq!(hand_amount("all_trump", &facts), 0);
    facts.opening_hand.card_count = 0;
    facts.opening_hand.trump_count = 0;
    assert_eq!(hand_amount("all_trump", &facts), 0);
}

#[test]
fn kitty_conditions_distinguish_capture_raw_points_and_penalties() {
    let mut facts = statistics(1);
    facts.result.kitty_points = 10;
    facts.result.kitty_multiplier = 8;
    facts.last_trick_winner = facts.player;
    facts.last_winning_play.components = vec![Component::Tractor {
        cards: vec![],
        pair_count: 2,
        top_strength: 10,
    }];
    assert_eq!(hand_amount("structured_kitty", &facts), 1);
    let card = Card::small_joker(0);
    for (component, expected) in [
        (Component::Single { card, strength: 20 }, 0),
        (
            Component::Pair {
                cards: [card; 2],
                strength: 20,
            },
            0,
        ),
        (
            Component::Triple {
                cards: [card; 3],
                strength: 20,
            },
            0,
        ),
        (
            Component::Quad {
                cards: [card; 4],
                strength: 20,
            },
            1,
        ),
        (
            Component::Tractor {
                cards: vec![],
                pair_count: 2,
                top_strength: 20,
            },
            1,
        ),
    ] {
        facts.last_winning_play.components = vec![component];
        assert_eq!(hand_amount("structured_kitty", &facts), expected);
    }

    facts.last_trick_winner = ShengjiPlayerId(3);
    assert_eq!(hand_amount("structured_kitty", &facts), 0);
    facts.last_trick_winner = facts.player;
    facts.result.kitty_points = 0;
    assert_eq!(hand_amount("structured_kitty", &facts), 0);
    facts.result.trick_points = 200;
    assert_eq!(hand_amount("all_points", &facts), 1);
    facts.result.trick_points = 195;
    facts.result.collecting_score = 200;
    facts.result.penalty_adjustment = 5;
    assert_eq!(hand_amount("all_points", &facts), 0);
    facts.result.trick_points = 0;
    facts.result.kitty_points = 10;
    facts.result.collecting_score = 80;
    facts.result.promoted_team = ShengjiTeamId(1);
    facts.result.penalty_adjustment = 0;
    assert_eq!(hand_amount("kitty_takeover", &facts), 1);
    facts.result.trick_points = 5;
    facts.result.penalty_adjustment = -5;
    assert_eq!(hand_amount("kitty_takeover", &facts), 0);
    facts.result.trick_points = 0;
    facts.result.penalty_adjustment = 5;
    assert_eq!(hand_amount("kitty_takeover", &facts), 0);
}

#[test]
fn consecutive_shutouts_reset_after_any_other_result() {
    let mut facts = statistics(0);
    facts.result.collecting_score = 0;
    let mut game = ShengjiMatchStatistics::default();
    for count in 1..=3 {
        game.record_hand(&facts.result);
        let mut trigger = analyzed(facts.clone());
        let AchievementTrigger::Game {
            event:
                GameEvent::Shengji(ShengjiEvent::HandAnalyzed {
                    match_statistics, ..
                }),
            ..
        } = &mut trigger
        else {
            unreachable!()
        };
        *match_statistics = game;
        assert_eq!(amount("three_shutouts", &trigger), u64::from(count == 3));
    }
    facts.result.collecting_score = 5;
    game.record_hand(&facts.result);
    assert_eq!(game.consecutive_dealer_shutouts, [0, 0]);
    facts.result.collecting_score = 0;
    game.record_hand(&facts.result);
    assert_eq!(game.consecutive_dealer_shutouts, [1, 0]);
    facts.result.dealer_team = ShengjiTeamId(1);
    game.record_hand(&facts.result);
    assert_eq!(game.consecutive_dealer_shutouts, [0, 1]);
}
