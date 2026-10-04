use super::support::*;
use leocard_achievements::{AchievementCategory, AchievementTier, achievements_in};

#[test]
fn all_twenty_eight_conditions_require_the_named_local_players_facts() {
    let entries = achievements_in(AchievementCategory::QiGui523).collect::<Vec<_>>();
    assert_eq!(entries.len(), 28);
    for (tier, count) in [
        (AchievementTier::Bronze, 15),
        (AchievementTier::Silver, 8),
        (AchievementTier::Gold, 5),
    ] {
        assert_eq!(
            entries
                .iter()
                .filter(|definition| definition.tier == tier)
                .count(),
            count
        );
    }
    let mut s = statistics();
    s.progress.first_trick_with_points = true;
    s.progress.consecutive_pair_plays = 1;
    s.progress.bomb_plays = 10;
    s.progress.heaven_bomb_plays = 1;
    s.progress.airplane_plays = 1;
    s.progress.max_silent_tricks = 5;
    s.progress.won_scoring_bomb = true;
    s.progress.max_unanswered_tricks = 5;
    s.progress.full_high_hand = true;
    s.progress.max_straight_run = 6;
    s.progress.max_trick_points = 100;
    s.diamond_four = true;
    s.spade_seven_follow = true;
    s.heaven_over_heaven = true;
    s.late_bomb = true;
    s.straight_from_four = true;
    s.full_hand_play = true;
    s.bomb_revenge = true;
    s.comeback = true;
    s.won = true;
    s.all_points = true;
    s.uncontested_win = true;
    s.played_count = 12;
    s.captured_fives = 3;
    s.captured_tens_and_kings = 4;
    s.captured_points = 55;
    let positive = trigger(s.clone());
    let empty = trigger(statistics());
    let mut wrong_player = positive.clone();
    let AchievementTrigger::Game { player, .. } = &mut wrong_player else {
        unreachable!()
    };
    *player = PlayerId(1);
    let mut mismatched = s;
    mismatched.player = QiGuiPlayerId(1);
    let mismatched = trigger(mismatched);
    for definition in entries {
        let amount = definition.criteria[0].amount;
        assert!(amount(&positive) > 0, "{}", definition.title);
        assert_eq!(amount(&empty), 0, "{}", definition.title);
        assert_eq!(amount(&wrong_player), 0, "{}", definition.title);
        assert_eq!(amount(&mismatched), 0, "{}", definition.title);
    }
}

#[test]
fn runs_and_trick_thresholds_do_not_unlock_one_step_early() {
    let mut s = statistics();
    s.progress.bomb_plays = 4;
    s.progress.max_silent_tricks = 4;
    s.progress.max_unanswered_tricks = 4;
    s.progress.max_straight_run = 5;
    s.progress.max_trick_points = 99;
    s.played_count = 11;
    for id in [
        "five_bombs",
        "ten_bombs",
        "five_silent_tricks",
        "five_unanswered_tricks",
        "six_straights",
        "hundred_point_trick",
        "twelve_cards",
    ] {
        assert_eq!(amount(id, s.clone()), 0, "{id}");
    }
    s.progress.heaven_bomb_plays = 1;
    assert_eq!(amount("five_bombs", s.clone()), 1);
    assert_eq!(amount("ten_bombs", s.clone()), 0);
    s.progress.bomb_plays = 9;
    assert_eq!(amount("ten_bombs", s), 1);
}
