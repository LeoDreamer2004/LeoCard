use super::support::*;
use leocard_achievements::{AchievementCategory, AchievementTier, achievements_in};

#[test]
fn all_twenty_seven_awards_match_only_their_own_positive_facts() {
    let entries = achievements_in(AchievementCategory::Uno).collect::<Vec<_>>();
    assert_eq!(entries.len(), 27);
    for (tier, count) in [
        (AchievementTier::Bronze, 16),
        (AchievementTier::Silver, 6),
        (AchievementTier::Gold, 5),
    ] {
        assert_eq!(
            entries.iter().filter(|entry| entry.tier == tier).count(),
            count
        );
    }
    type Case = (&'static str, fn(&mut UnoActionStatistics));
    let cases: &[Case] = &[
        ("call_uno", |s| s.progress.uno_calls = 1),
        ("report_uno", |s| s.progress.uno_reports = 1),
        ("pair_win", |s| {
            s.rules.jump_in = true;
            s.won = true;
            s.progress.finished_with_pair = true;
        }),
        ("all_numbers", |s| s.progress.numbers = 0x03ff),
        ("dark_win", |s| {
            s.rules.mode = Mode::Flip;
            s.won = true;
            s.dark_side = true;
        }),
        ("eliminated", |s| {
            s.rules.mode = Mode::NoMercy;
            s.eliminated = true;
        }),
        ("sole_survivor", |s| {
            s.rules.mode = Mode::NoMercy;
            s.won = true;
            s.all_opponents_eliminated = true;
        }),
        ("draw_backfire", |s| s.progress.draw_chain_returned = true),
        ("three_skips", |s| {
            s.rules.action_stacking = true;
            s.progress.max_skip_batch = 3;
        }),
        ("jump_in", |s| s.progress.jump_ins = 1),
        ("twenty_four", |s| {
            s.rules.mode = Mode::NoMercy;
            s.progress.reached_twenty_four = true;
        }),
        ("challenge", |s| s.progress.successful_challenges = 1),
        ("hand_swaps", |s| {
            s.rules.mode = Mode::NoMercy;
            s.hand_swaps = 1;
        }),
        ("no_penalty_win", |s| s.won = true),
        ("jump_reverse", |s| s.progress.jumped_reverse = true),
        ("color_gift", |s| {
            s.won = true;
            s.progress.finished_with_color_gift = true;
        }),
        ("wild_two_hundred", |s| s.wild_cards = 1),
        ("hundred_jumps", |s| s.jump_ins = 1),
        ("no_draw_win", |s| s.won = true),
        ("forgotten_uno_win", |s| {
            s.won = true;
            s.progress.finished_without_uno = true;
        }),
        ("fifty_cards", |s| s.progress.peak_hand = 50),
        ("lucky_elimination", |s| {
            s.rules.mode = Mode::NoMercy;
            s.progress.lucky_elimination = true;
        }),
        ("one_color_win", |s| {
            s.won = true;
            s.progress.colors = 1;
        }),
        ("ten_thousand", |s| s.completed_game = true),
        ("full_draw_chain", |s| s.progress.full_draw_chain = true),
        ("ten_uno_turns", |s| s.progress.max_uno_call_run = 10),
        ("merciful_win", |s| {
            s.rules.mode = Mode::NoMercy;
            s.won = true;
        }),
    ];
    for (id, set) in cases {
        assert_eq!(amount(id, statistics()), 0, "{id}");
        let mut stats = statistics();
        set(&mut stats);
        assert_eq!(amount(id, stats.clone()), 1, "{id}");
        let mut wrong_player = trigger(stats);
        let AchievementTrigger::Game { player, .. } = &mut wrong_player else {
            unreachable!()
        };
        *player = PlayerId(1);
        assert_eq!(
            (achievement_by_id(&format!("leocard:uno/{id}"))
                .unwrap()
                .criteria[0]
                .amount)(&wrong_player),
            0
        );
    }
}

#[test]
fn winning_conditions_do_not_accept_losses_wrong_modes_or_penalties() {
    let mut stats = statistics();
    stats.won = true;
    stats.progress.has_penalty = true;
    stats.progress.has_drawn = true;
    stats.progress.colors = 3;
    assert_eq!(amount("no_penalty_win", stats.clone()), 0);
    assert_eq!(amount("no_draw_win", stats.clone()), 0);
    assert_eq!(amount("one_color_win", stats.clone()), 0);
    stats.rules.mode = Mode::NoMercy;
    stats.progress.peak_hand = 50;
    stats.progress.played_penalty = true;
    assert_eq!(amount("fifty_cards", stats.clone()), 0);
    assert_eq!(amount("merciful_win", stats.clone()), 0);
    assert_eq!(amount("sole_survivor", stats), 0);
}
