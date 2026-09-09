use super::*;

#[test]
fn dealer_may_bury_any_eight_cards() {
    let mut game = dealt_game(ShengjiRuleSet::default());
    let dealer = game.dealer().unwrap();
    let buried = game.players()[usize::from(dealer.0)].hand[..8].to_vec();
    game.bury(dealer, &buried).unwrap();
    assert_eq!(game.buried(), buried);
    assert_eq!(game.bottom_burier(), Some(dealer));
    assert_eq!(game.players()[usize::from(dealer.0)].hand.len(), 25);
    assert_eq!(game.current_player(), Some(dealer));
}

#[test]
fn constant_trump_first_hand_starts_both_teams_at_three() {
    let rules = ShengjiRuleSet {
        constant_trump: true,
        ..ShengjiRuleSet::default()
    };
    let game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();

    assert_eq!(
        game.teams().levels,
        [ShengjiRank::Three, ShengjiRank::Three]
    );
    assert_eq!(game.bidding().level(), ShengjiRank::Three);
}

#[test]
fn dealer_throw_penalty_adds_to_collectors_and_collectors_never_go_below_zero() {
    let mut game = dealt_game(ShengjiRuleSet {
        throw_penalty: ShengjiThrowPenalty::TenPerCard,
        ..ShengjiRuleSet::default()
    });
    game.dealer = Some(ShengjiPlayerId(0));
    game.apply_throw_penalty(ShengjiTeamId(0), 50);
    assert_eq!(game.collecting_score(), 50);
    game.apply_throw_penalty(ShengjiTeamId(1), 80);
    assert_eq!(game.collecting_score(), 0);
    game.apply_throw_penalty(ShengjiTeamId(1), 10);
    assert_eq!(game.collecting_score(), 0);
}

#[test]
fn failed_throw_keeps_the_attempt_visible_but_only_plays_the_forced_low_card() {
    let mut game = GameState::standard(build_deck()).unwrap();
    let low = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Three);
    let high = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Nine);
    let beating_card = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Four);
    game.rules.throw_penalty = ShengjiThrowPenalty::TenPerCard;
    game.phase = Phase::Playing;
    game.trump = Some(ShengjiTrump::new(ShengjiRank::Two, Some(ShengjiSuit::Spade)).unwrap());
    game.dealer = Some(ShengjiPlayerId(0));
    game.current_player = Some(ShengjiPlayerId(0));
    game.players[0].hand = vec![low, high];
    game.players[1].hand = vec![beating_card];

    let attempted = vec![high, low];
    let outcome = game.play_cards(ShengjiPlayerId(0), &attempted).unwrap();
    let ActionOutcome::ThrowFailed {
        player,
        attempted: visible_attempt,
        forced,
        penalty_points,
        next,
    } = outcome
    else {
        panic!("the beatable mixed single throw should fail");
    };

    assert_eq!(player, ShengjiPlayerId(0));
    assert_eq!(visible_attempt, attempted);
    assert_eq!(forced.cards, vec![low]);
    assert_eq!(penalty_points, 20);
    assert_eq!(next, ShengjiPlayerId(1));
    assert_eq!(game.players[0].hand, vec![high]);
    assert_eq!(game.collecting_score(), 20);
    assert_eq!(game.current_player(), Some(ShengjiPlayerId(1)));
}

#[test]
fn settlement_thresholds_and_next_dealers_match_the_declared_rules() {
    let cases = [
        (0, ShengjiTeamId(0), 3, ShengjiPlayerId(2)),
        (35, ShengjiTeamId(0), 2, ShengjiPlayerId(2)),
        (75, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (80, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (120, ShengjiTeamId(1), 1, ShengjiPlayerId(1)),
        (160, ShengjiTeamId(1), 2, ShengjiPlayerId(1)),
        (200, ShengjiTeamId(1), 3, ShengjiPlayerId(1)),
    ];
    for (score, promoted_team, steps, next_dealer) in cases {
        let mut game = dealt_game(ShengjiRuleSet::default());
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team);
        assert_eq!(result.promoted_steps, steps);
        assert_eq!(result.next_dealer, next_dealer);
    }
}

#[test]
fn three_deck_settlement_uses_sixty_point_bands() {
    let cases = [
        (0, ShengjiTeamId(0), 3, ShengjiPlayerId(2)),
        (59, ShengjiTeamId(0), 2, ShengjiPlayerId(2)),
        (60, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (119, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (120, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (179, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (180, ShengjiTeamId(1), 1, ShengjiPlayerId(1)),
        (240, ShengjiTeamId(1), 2, ShengjiPlayerId(1)),
    ];
    for (score, promoted_team, steps, next_dealer) in cases {
        let mut game = dealt_game(ShengjiRuleSet {
            deck_count: 3,
            ..ShengjiRuleSet::default()
        });
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team, "score={score}");
        assert_eq!(result.promoted_steps, steps, "score={score}");
        assert_eq!(result.next_dealer, next_dealer, "score={score}");
    }
}

#[test]
fn four_deck_settlement_uses_eighty_point_bands() {
    let cases = [
        (0, ShengjiTeamId(0), 3),
        (79, ShengjiTeamId(0), 2),
        (80, ShengjiTeamId(0), 1),
        (159, ShengjiTeamId(0), 1),
        (160, ShengjiTeamId(1), 0),
        (239, ShengjiTeamId(1), 0),
        (240, ShengjiTeamId(1), 1),
        (320, ShengjiTeamId(1), 2),
    ];
    for (score, promoted_team, steps) in cases {
        let mut game = dealt_game(ShengjiRuleSet {
            deck_count: 4,
            ..ShengjiRuleSet::default()
        });
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team, "score={score}");
        assert_eq!(result.promoted_steps, steps, "score={score}");
    }
}
