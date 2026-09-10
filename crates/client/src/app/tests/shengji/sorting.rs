use super::prelude::*;

#[test]
fn shengji_turn_sync_preselects_the_only_required_pair() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let pair = |rank| {
        [
            ShengjiCard::suited(0, ShengjiSuit::Spade, rank),
            ShengjiCard::suited(1, ShengjiSuit::Spade, rank),
        ]
    };
    let threes = pair(ShengjiRank::Three);
    let hand = [
        threes.as_slice(),
        &[
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Six),
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Seven),
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Nine),
        ],
    ]
    .concat();
    let lead_cards = [
        pair(ShengjiRank::Jack).as_slice(),
        pair(ShengjiRank::Queen).as_slice(),
    ]
    .concat();
    let TrickPlay::Accepted(lead) =
        classify_lead(&lead_cards, trump, &ShengjiRuleSet::default(), &[]).unwrap()
    else {
        unreachable!("a single tractor is not a throw")
    };
    let mut game = shengji_ui_snapshot(hand, None);
    game.phase = ShengjiPhaseView::Playing;
    game.trump = Some(trump);
    game.current_player = Some(game.you);
    game.trick = Some(ShengjiTrickView {
        leader: PlayerId(1),
        current_player: game.you,
        winning_player: PlayerId(1),
        plays: vec![ShengjiPublicPlay {
            player: PlayerId(1),
            play: lead,
            throw_penalty: 0,
        }],
        table_points: 0,
    });
    let mut ui = ShengjiUiState::default();

    select_forced_shengji_follow_cards(&game, &mut ui);

    assert_eq!(ui.selected, threes.into_iter().collect());

    let first = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = first.iter().copied().collect();
    let second = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = second.iter().copied().collect();
    let third = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = third.iter().copied().collect();
    let wrapped = next_shengji_hint(&game, &ui.selected).unwrap();

    assert_ne!(first, second);
    assert_ne!(second, third);
    assert_eq!(wrapped, first);
}

#[test]
fn shengji_hand_sort_keeps_all_trumps_before_side_suits() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let big = ShengjiCard::big_joker(0);
    let main_level = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let off_level = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let side_ace = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let mut cards = vec![side_ace, off_level, main_level, big];

    sort_shengji_cards(&mut cards, Some(trump));

    assert_eq!(cards, vec![big, main_level, off_level, side_ace]);
}

#[test]
fn shengji_hand_sort_places_unbid_level_cards_immediately_after_jokers() {
    let big = ShengjiCard::big_joker(0);
    let small = ShengjiCard::small_joker(0);
    let spade = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let heart = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let club = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten);
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten);
    let side_ace = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let game = shengji_ui_snapshot(Vec::new(), None);
    let mut cards = vec![side_ace, diamond, small, club, big, heart, spade];

    assert_eq!(shengji_display_trump(&game), None);
    sort_shengji_cards(&mut cards, shengji_hand_sort_trump(&game));

    assert_eq!(
        cards,
        vec![big, small, spade, heart, club, diamond, side_ace]
    );
}

#[test]
fn shengji_hand_sort_groups_off_suit_level_pairs_in_spade_heart_club_diamond_order() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, None).unwrap();
    let spades = [
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten),
    ];
    let hearts = [
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let clubs = [
        ShengjiCard::suited(1, ShengjiSuit::Club, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten),
    ];
    let diamonds = [
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
    ];
    let mut cards = vec![
        diamonds[0],
        spades[1],
        hearts[0],
        clubs[1],
        spades[0],
        diamonds[1],
        clubs[0],
        hearts[1],
    ];

    sort_shengji_cards(&mut cards, Some(trump));

    assert_eq!(
        cards,
        [spades, hearts, clubs, diamonds]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    );
}
