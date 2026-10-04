use super::support::*;

#[test]
fn first_trick_and_unanswered_runs_use_completed_tricks_not_pass_actions() {
    let hand = (0..6).map(|d| card(d, QiGuiRank::Five)).collect::<Vec<_>>();
    let mut f = Fixture::new(
        vec![
            hand.clone(),
            bomb(0, QiGuiRank::King),
            bomb(0, QiGuiRank::Ten),
        ],
        Vec::new(),
        6,
    );
    for (index, card) in hand.iter().enumerate() {
        let played = f.play(0, &[*card]);
        assert_eq!(
            played[0].progress.max_unanswered_tricks,
            index as u32 + u32::from(played[0].completed_game)
        );
        if index == 5 {
            break;
        }
        let partial = f.pass(2);
        assert_eq!(partial[1].progress.max_silent_tricks, index as u32);
        let completed = f.pass(1);
        assert_eq!(completed[0].captured_fives, 1);
        assert_eq!(completed[0].captured_points, 5);
        assert_eq!(
            completed[0].progress.max_unanswered_tricks,
            index as u32 + 1
        );
        assert_eq!(completed[1].progress.max_silent_tricks, index as u32 + 1);
        assert!(completed[0].progress.first_trick_with_points);
    }
    assert_eq!(f.statistics.players()[0].max_unanswered_tricks, 6);
    assert_eq!(f.statistics.players()[2].max_silent_tricks, 6);
}

#[test]
fn first_zero_point_trick_does_not_become_first_blood_later_and_reply_breaks_runs() {
    let mut f = Fixture::new(
        vec![
            vec![card(0, QiGuiRank::Four), card(0, QiGuiRank::Five)],
            vec![card(0, QiGuiRank::Seven), card(0, QiGuiRank::King)],
        ],
        Vec::new(),
        5,
    );
    f.play(0, &[card(0, QiGuiRank::Four)]);
    let first = f.pass(1);
    assert!(!first[0].progress.first_trick_with_points);
    f.play(0, &[card(0, QiGuiRank::Five)]);
    // The game ends immediately here: remaining cards are not a 100-point trick.
    assert_eq!(f.statistics.players()[0].max_trick_points, 5);
    assert!(!f.statistics.players()[0].first_trick_with_points);

    let mut f = Fixture::new(
        vec![
            vec![card(0, QiGuiRank::Four), card(0, QiGuiRank::Six)],
            vec![card(0, QiGuiRank::Eight), card(0, QiGuiRank::Nine)],
        ],
        vec![card(0, QiGuiRank::Ten)],
        5,
    );
    f.play(0, &[card(0, QiGuiRank::Four)]);
    f.play(1, &[card(0, QiGuiRank::Eight)]);
    f.pass(0);
    assert!(
        f.statistics
            .players()
            .iter()
            .all(|p| p.max_silent_tricks == 0 && p.max_unanswered_tricks == 0)
    );
}

#[test]
fn hundred_point_trick_counts_physical_score_cards_and_final_collection_is_separate() {
    let ten = [bomb(0, QiGuiRank::Ten), bomb(1, QiGuiRank::Ten)].concat();
    let five = [bomb(0, QiGuiRank::Five), bomb(1, QiGuiRank::Five)].concat();
    let mut hand = ten.clone();
    hand.push(card(2, QiGuiRank::Four));
    let mut other = five.clone();
    other.push(card(2, QiGuiRank::Six));
    let mut f = Fixture::new(vec![hand, other], Vec::new(), 9);
    f.play(0, &ten);
    let eight_fives = [bomb(0, QiGuiRank::Five), bomb(1, QiGuiRank::Five)].concat();
    f.play(1, &eight_fives);
    let settled = f.pass(0);
    assert_eq!(settled[1].captured_points, 120);
    assert_eq!(settled[1].captured_fives, 8);
    assert_eq!(settled[1].captured_tens_and_kings, 8);
    assert_eq!(settled[1].progress.max_trick_points, 120);
}

#[test]
fn last_play_comeback_and_remaining_score_cards_are_reported_once() {
    let mut f = Fixture::new(
        vec![
            vec![card(0, QiGuiRank::Seven)],
            vec![card(0, QiGuiRank::Five), card(0, QiGuiRank::King)],
        ],
        Vec::new(),
        5,
    );
    f.game.players[1].score = 10;
    // Recreate the observer from a consistent state including already captured scores.
    f.statistics = QiGuiMatchStatistics::new(&f.game);
    let before = QiGuiActionContext::capture(&f.game, QiGuiPlayerId(0)).unwrap();
    let replay = QiGuiActionContext::capture(&f.game, QiGuiPlayerId(0)).unwrap();
    let outcome = f
        .game
        .play_cards(QiGuiPlayerId(0), &[card(0, QiGuiRank::Seven)])
        .unwrap();
    let play = classify(&[card(0, QiGuiRank::Seven)], f.game.rules()).unwrap();
    let reports = f.statistics.observe(before, &outcome, Some(&play), &f.game);
    assert!(reports[0].won && reports[0].comeback && reports[0].uncontested_win);
    assert_eq!(reports[0].captured_points, 15);
    assert_eq!(reports[0].captured_fives, 1);
    assert_eq!(reports[0].captured_tens_and_kings, 1);
    assert_eq!(reports[0].progress.max_trick_points, 0);
    assert!(!reports[0].all_points);
    assert!(reports.iter().all(|s| s.completed_game));
    assert!(
        f.statistics
            .observe(replay, &outcome, Some(&play), &f.game)
            .is_empty()
    );
}

#[test]
fn full_high_hand_uses_game_strength_and_is_not_a_short_final_hand() {
    let high = [
        QiGuiRank::Three,
        QiGuiRank::Two,
        QiGuiRank::Five,
        QiGuiRank::Joker,
        QiGuiRank::Seven,
    ]
    .map(|rank| card(0, rank));
    let mut f = Fixture::new(
        vec![high.to_vec(), straight(1)],
        vec![card(0, QiGuiRank::Four)],
        5,
    );
    assert!(f.play(0, &[high[0]])[0].progress.full_high_hand);
    let mut f = Fixture::new(
        vec![vec![card(0, QiGuiRank::Seven)], straight(1)],
        Vec::new(),
        5,
    );
    assert!(
        !f.play(0, &[card(0, QiGuiRank::Seven)])[0]
            .progress
            .full_high_hand
    );
}
