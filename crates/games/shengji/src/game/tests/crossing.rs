use super::*;

fn five_trump_crossing_game(trump_suit: Option<ShengjiSuit>) -> (GameState, Vec<ShengjiCard>) {
    let rules = ShengjiRuleSet {
        five_trump_crossing: true,
        ..ShengjiRuleSet::default()
    };
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    let buried = [
        ShengjiRank::Three,
        ShengjiRank::Four,
        ShengjiRank::Five,
        ShengjiRank::Six,
        ShengjiRank::Seven,
        ShengjiRank::Eight,
        ShengjiRank::Nine,
        ShengjiRank::Jack,
    ]
    .into_iter()
    .map(|rank| ShengjiCard::suited(0, ShengjiSuit::Diamond, rank))
    .collect::<Vec<_>>();
    let player_zero = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::big_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Six),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Seven),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Eight),
    ];
    let player_two = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Five),
        ShengjiCard::small_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Five),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Six),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Seven),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Eight),
    ];
    game.players[0].hand = [buried.as_slice(), player_zero.as_slice()].concat();
    game.players[1].hand = [
        ShengjiRank::Six,
        ShengjiRank::Seven,
        ShengjiRank::Eight,
        ShengjiRank::Nine,
        ShengjiRank::Jack,
        ShengjiRank::Queen,
    ]
    .into_iter()
    .map(|rank| ShengjiCard::suited(0, ShengjiSuit::Heart, rank))
    .collect();
    game.players[2].hand = player_two.to_vec();
    game.players[3].hand = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::King),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ace),
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten),
        ShengjiCard::big_joker(1),
    ];
    game.trump = Some(ShengjiTrump::new(ShengjiRank::Ten, trump_suit).unwrap());
    game.dealer = Some(ShengjiPlayerId(0));
    game.phase = Phase::Burying;
    (game, buried)
}

#[test]
fn teammates_can_cross_each_other_once_with_both_transfers_applied_simultaneously() {
    let (mut game, buried) = five_trump_crossing_game(Some(ShengjiSuit::Heart));
    game.bury(ShengjiPlayerId(0), &buried).unwrap();
    assert_eq!(game.phase(), &Phase::FiveTrumpCrossing);
    let state = game.five_trump_crossing().unwrap();
    assert!(state.eligible(ShengjiPlayerId(0)));
    assert!(state.eligible(ShengjiPlayerId(2)));
    assert!(!state.eligible(ShengjiPlayerId(1)));
    assert!(!state.eligible(ShengjiPlayerId(3)));
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(1), None),
        Err(GameError::CrossingNotEligible)
    );

    let initial_zero = game.players[0].hand.clone();
    let initial_two = game.players[2].hand.clone();
    let zero_out = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::big_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
    ];
    let invalid_zero = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five),
    ];
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(0), Some(&invalid_zero)),
        Err(GameError::CrossingMustIncludeAllTrumps)
    );
    game.choose_five_trump_crossing(ShengjiPlayerId(0), Some(&zero_out))
        .unwrap();
    assert_eq!(game.players[0].hand, initial_zero, "决定阶段不能提前移动牌");

    let two_out = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Five),
        ShengjiCard::small_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Five),
    ];
    game.choose_five_trump_crossing(ShengjiPlayerId(2), Some(&two_out))
        .unwrap();
    assert_eq!(
        game.five_trump_crossing().unwrap().stage(),
        FiveTrumpCrossingStage::Returning
    );
    assert_eq!(game.players[0].hand.len(), initial_zero.len());
    assert_eq!(game.players[2].hand.len(), initial_two.len());

    let after_outgoing_zero = game.players[0].hand.clone();
    game.return_five_trump_crossing(ShengjiPlayerId(0), &two_out)
        .unwrap();
    assert_eq!(
        game.players[0].hand, after_outgoing_zero,
        "回牌也必须等所有人选定后再同时移动"
    );
    game.return_five_trump_crossing(ShengjiPlayerId(2), &zero_out)
        .unwrap();
    assert_eq!(game.phase(), &Phase::Playing);

    let mut final_zero = game.players[0].hand.clone();
    let mut final_two = game.players[2].hand.clone();
    let mut initial_zero = initial_zero;
    let mut initial_two = initial_two;
    for hand in [
        &mut final_zero,
        &mut final_two,
        &mut initial_zero,
        &mut initial_two,
    ] {
        hand.sort_by(ShengjiCard::identity_cmp);
    }
    assert_eq!(final_zero, initial_zero);
    assert_eq!(final_two, initial_two);
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(0), None),
        Err(GameError::WrongPhase),
        "一局只能进行一次五主过江"
    );
}

#[test]
fn five_trump_crossing_is_skipped_in_no_trump_and_all_declines_start_play() {
    let (mut no_trump, buried) = five_trump_crossing_game(None);
    no_trump.bury(ShengjiPlayerId(0), &buried).unwrap();
    assert_eq!(no_trump.phase(), &Phase::Playing);
    assert!(no_trump.five_trump_crossing().is_none());

    let (mut suited, buried) = five_trump_crossing_game(Some(ShengjiSuit::Heart));
    suited.bury(ShengjiPlayerId(0), &buried).unwrap();
    suited
        .choose_five_trump_crossing(ShengjiPlayerId(0), None)
        .unwrap();
    suited
        .choose_five_trump_crossing(ShengjiPlayerId(2), None)
        .unwrap();
    assert_eq!(suited.phase(), &Phase::Playing);
    assert_eq!(suited.current_player(), suited.dealer());
}
