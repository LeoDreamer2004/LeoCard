use super::support::*;
use leocard_achievements::AchievementContext;
use leocard_protocol::MatchId;

#[test]
fn lifetime_counters_use_deltas_and_preserve_receipts_across_reload() {
    let mut book = AchievementBook::default();
    for count in 1..=200 {
        let mut stats = statistics();
        stats.progress.wild_cards = count;
        stats.wild_cards = 1;
        stats.jump_ins = 1;
        stats.hand_swaps = 1;
        stats.rules.mode = Mode::NoMercy;
        let event = trigger(stats);
        let context = Some(AchievementContext {
            match_id: MatchId([1; 16]),
            hand_index: None,
            sequence: u128::from(count),
        });
        book.trigger(&event, context, 100);
        if count == 99 {
            book = postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
        }
        assert!(!book.trigger(&event, context, 100).progressed);
        for (id, threshold) in [
            ("wild_two_hundred", 200),
            ("hundred_jumps", 100),
            ("hand_swaps", 10),
        ] {
            assert_eq!(
                book.achieved(achievement_by_id(&format!("leocard:uno/{id}")).unwrap()),
                count >= threshold
            );
        }
    }
}

#[test]
fn only_completed_games_count_and_partial_game_masks_do_not_combine() {
    let mut book = AchievementBook::default();
    let completed = achievement_by_id("leocard:uno/ten_thousand").unwrap();
    for count in 1_u32..=10_000 {
        let mut stats = statistics();
        stats.progress.numbers = if count % 2 == 0 { 0x1f } else { 0x3e0 };
        book.trigger(&trigger(stats.clone()), None, 10);
        assert_eq!(
            book.criterion_count(completed, "progress"),
            u64::from(count - 1)
        );
        stats.completed_game = true;
        let context = Some(AchievementContext {
            match_id: MatchId([2; 16]),
            hand_index: None,
            sequence: u128::from(count),
        });
        book.trigger(&trigger(stats.clone()), context, 20);
        assert!(!book.trigger(&trigger(stats), context, 20).progressed);
        assert_eq!(book.achieved(completed), count == 10_000);
    }
    assert!(!book.achieved(achievement_by_id("leocard:uno/all_numbers").unwrap()));
}
