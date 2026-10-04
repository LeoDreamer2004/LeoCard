use super::support::*;

#[test]
fn failed_actions_do_not_count_and_identical_sevens_require_follow_policy() {
    let seven = card(0, QiGuiRank::Seven);
    let other = card(1, QiGuiRank::Seven);
    let mut f = Fixture::new(
        vec![vec![seven], vec![other]],
        vec![card(2, QiGuiRank::Four)],
        5,
    );
    assert!(f.action(1, Some(&[other])).is_err());
    assert_eq!(f.statistics.players()[1].plays, 0);
    assert!(!f.play(0, &[seven])[0].spade_seven_follow);
    assert!(f.action(1, Some(&[other])).is_err());
    assert_eq!(f.statistics.players()[1].plays, 0);
    f.game.rules.same_card_policy = SameCardPolicy::CanFollow;
    assert!(f.play(1, &[other])[1].spade_seven_follow);
}

#[test]
fn accepted_shapes_full_hand_and_late_bombs_use_actual_pre_action_state() {
    let eight = [bomb(0, QiGuiRank::Four), bomb(1, QiGuiRank::Four)].concat();
    let mut f = Fixture::new(vec![eight.clone(), bomb(2, QiGuiRank::Five)], Vec::new(), 8);
    let s = f.play(0, &eight).remove(0);
    assert!(s.full_hand_play && s.late_bomb);
    assert_eq!(s.progress.bomb_plays, 1);
    assert!(s.completed_game && s.won && s.all_points && s.uncontested_win);
    let mut f = Fixture::new(vec![eight.clone(), bomb(2, QiGuiRank::Five)], Vec::new(), 9);
    assert!(!f.play(0, &eight)[0].full_hand_play);
    let four = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four);
    let mut f = Fixture::new(
        vec![vec![four], vec![card(0, QiGuiRank::Five)]],
        vec![card(0, QiGuiRank::Six)],
        5,
    );
    assert!(f.play(0, &[four])[0].diamond_four);
    let heaven = heaven(0, QiGuiSuit::Spade);
    let mut f = Fixture::new(
        vec![heaven.to_vec(), vec![card(0, QiGuiRank::King)]],
        Vec::new(),
        5,
    );
    let s = f.play(0, &heaven).remove(0);
    assert!(s.late_bomb);
    assert_eq!(s.progress.heaven_bomb_plays, 1);
}

#[test]
fn six_own_straight_turns_count_but_an_own_pass_breaks_the_run() {
    let mut pile = Vec::new();
    for deck in 1..6 {
        pile.extend(straight(deck));
    }
    let mut f = Fixture::new(
        vec![
            straight(0),
            [bomb(6, QiGuiRank::Ace), vec![card(6, QiGuiRank::Seven)]].concat(),
        ],
        pile,
        5,
    );
    for deck in 0..6 {
        let s = f.play(0, &straight(deck)).remove(0);
        assert!(s.straight_from_four);
        assert_eq!(s.progress.max_straight_run, u32::from(deck) + 1);
        if deck < 5 {
            f.pass(1);
        }
    }
    assert!(matches!(f.game.phase(), Phase::Finished(_)));
    assert_eq!(f.statistics.players()[0].straight_plays, 6);
    let ranks = [
        QiGuiRank::Six,
        QiGuiRank::Eight,
        QiGuiRank::Nine,
        QiGuiRank::Ten,
        QiGuiRank::Jack,
    ];
    let upper = ranks.map(|r| card(7, r));
    let mut f = Fixture::new(
        vec![straight(0), upper.to_vec()],
        [straight(2), straight(3)].concat(),
        5,
    );
    f.play(0, &straight(0));
    f.play(1, &upper);
    f.pass(0);
    // A later new straight must start a fresh run after the accepted pass.
    f.game.players[0].hand = straight(1);
    f.game.trick = Some(TrickState::new(QiGuiPlayerId(0)));
    assert_eq!(f.play(0, &straight(1))[0].progress.max_straight_run, 1);
}

#[test]
fn scoring_bomb_chain_stays_within_one_trick_and_heaven_must_be_strictly_stronger() {
    let ten = bomb(0, QiGuiRank::Ten);
    let king = bomb(0, QiGuiRank::King);
    let five = bomb(0, QiGuiRank::Five);
    let mut f = Fixture::new(
        vec![ten.clone(), five.clone(), king.clone()],
        vec![card(0, QiGuiRank::Four)],
        5,
    );
    f.play(0, &ten);
    f.play(2, &king);
    assert!(f.play(1, &five)[1].bomb_revenge);
    f.pass(0);
    assert!(f.pass(2)[1].progress.won_scoring_bomb);
    let small = heaven(0, QiGuiSuit::Club);
    let big = heaven(1, QiGuiSuit::Spade);
    let mut f = Fixture::new(
        vec![small.clone(), big.clone()],
        vec![card(0, QiGuiRank::Four)],
        5,
    );
    f.play(0, &small);
    assert!(f.play(1, &big)[1].heaven_over_heaven);
    let mut f = Fixture::new(
        vec![big.clone(), heaven(2, QiGuiSuit::Spade)],
        vec![card(0, QiGuiRank::Four)],
        5,
    );
    f.game.rules.same_card_policy = SameCardPolicy::CanFollow;
    f.play(0, &big);
    let tied = f.game.players[1].hand.clone();
    assert!(!f.play(1, &tied)[1].heaven_over_heaven);
}
