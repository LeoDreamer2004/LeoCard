use super::support::*;
use leocard_achievements::AchievementContext;
use leocard_protocol::MatchId;

#[test]
fn captured_deltas_accumulate_across_games_and_reload_but_replays_do_not() {
    let mut book = AchievementBook::default();
    let mut s = statistics();
    s.captured_fives = 10;
    s.captured_tens_and_kings = 10;
    s.captured_points = 100;
    for index in 1..=100 {
        let context = Some(AchievementContext {
            match_id: MatchId([index; 16]),
            hand_index: None,
            sequence: 1,
        });
        let event = trigger(s.clone());
        book.trigger(&event, context, 100);
        assert!(!book.trigger(&event, context, 101).progressed);
        if index == 50 {
            book = postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
        }
        for id in ["thousand_fives", "thousand_tens", "ten_thousand_points"] {
            let definition = achievement_by_id(&format!("leocard:qigui523/{id}")).unwrap();
            assert_eq!(book.achieved(definition), index == 100);
        }
    }
}

#[test]
fn partial_match_runs_are_not_combined_across_games() {
    let mut book = AchievementBook::default();
    for index in 1..=10 {
        let mut s = statistics();
        s.progress.bomb_plays = 4;
        s.progress.max_straight_run = 5;
        s.progress.max_silent_tricks = 4;
        s.progress.max_unanswered_tricks = 4;
        book.trigger(
            &trigger(s),
            Some(AchievementContext {
                match_id: MatchId([index; 16]),
                hand_index: None,
                sequence: 1,
            }),
            100,
        );
    }
    for id in [
        "five_bombs",
        "ten_bombs",
        "six_straights",
        "five_silent_tricks",
        "five_unanswered_tricks",
    ] {
        assert!(!book.achieved(achievement_by_id(&format!("leocard:qigui523/{id}")).unwrap()));
    }
}
