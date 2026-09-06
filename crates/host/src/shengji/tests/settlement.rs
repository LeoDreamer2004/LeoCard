use super::*;
use leocard_shengji::ShengjiTeamId;
fn result_for_reference_points(
    dealer_wins: bool,
    promoted_steps: u8,
    collecting_score: u32,
) -> HandResult {
    let dealer_team = ShengjiTeamId(0);
    HandResult {
        dealer: ShengjiPlayerId(0),
        dealer_team,
        collecting_team: ShengjiTeamId(1),
        trick_points: collecting_score,
        penalty_adjustment: 0,
        kitty_points: 0,
        kitty_multiplier: 0,
        collecting_score,
        promoted_team: if dealer_wins {
            dealer_team
        } else {
            ShengjiTeamId(1)
        },
        promoted_steps,
        next_dealer: ShengjiPlayerId(1),
        levels: [ShengjiRank::Two, ShengjiRank::Two],
    }
}

#[test]
fn hand_rating_magnitude_matches_upgrade_outcome() {
    assert_eq!(
        finished_reference_point_magnitude(&result_for_reference_points(true, 3, 0)),
        6
    );
    assert_eq!(
        finished_reference_point_magnitude(&result_for_reference_points(true, 1, 40)),
        2
    );
    assert_eq!(
        finished_reference_point_magnitude(&result_for_reference_points(false, 0, 80)),
        2
    );
    assert_eq!(
        finished_reference_point_magnitude(&result_for_reference_points(false, 1, 120)),
        4
    );
    assert_eq!(
        finished_reference_point_magnitude(&result_for_reference_points(false, 2, 160)),
        6
    );
}

#[test]
fn hand_rating_is_applied_once_and_updates_every_player() {
    let (mut session, _, _) = started_session();
    session.statistics.profiles[0].declaration_games = 1;
    session.statistics.profiles[0].counter_games = 1;
    session.statistics.profiles[0].plays = 4;
    session.statistics.profiles[0].winning_plays = 2;
    let result = result_for_reference_points(false, 1, 120);
    session.apply_finished_reference_points(&result);
    let settlement_id = session.statistics.finished_settlement_id;
    let changes = session
        .statistics
        .finished_reference_changes
        .clone()
        .unwrap();

    assert!(settlement_id.is_some());
    assert_eq!(changes.len(), 4);
    for participant in &session.room.players {
        let expected = if participant.id.0 % 2 == 1 { 4 } else { -4 };
        assert_eq!(participant.reference_points, expected);
        assert_eq!(participant.completed_games, 1);
        let stats = participant.game_profiles.shengji.as_ref().unwrap();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(stats.total_reference_delta, i64::from(expected));
        if participant.id.0 % 2 == 0 {
            assert_eq!(stats.dealer_team_games, 1);
            assert_eq!(stats.dealer_team_score, 120);
            assert_eq!(stats.defended_kitty_games, 1);
        } else {
            assert_eq!(stats.collecting_team_games, 1);
            assert_eq!(stats.collecting_team_score, 120);
            assert_eq!(stats.captured_kitty_games, 0);
        }
        assert_eq!(stats.dealer_games, u32::from(participant.id == PlayerId(0)));
        assert_eq!(
            changes
                .iter()
                .find(|change| change.player == participant.id)
                .unwrap()
                .delta,
            expected as i16
        );
    }
    let dealer_stats = session.room.players[0]
        .game_profiles
        .shengji
        .as_ref()
        .unwrap();
    assert_eq!(dealer_stats.declaration_games, 1);
    assert_eq!(dealer_stats.counter_games, 1);
    assert_eq!(dealer_stats.plays, 4);
    assert_eq!(dealer_stats.winning_plays, 2);

    session.apply_finished_reference_points(&result);
    assert_eq!(session.statistics.finished_settlement_id, settlement_id);
    assert!(
        session
            .room
            .players
            .iter()
            .all(|participant| participant.completed_games == 1)
    );
}

#[test]
fn throw_profile_lengths_include_every_internal_sequence() {
    let (mut session, _, card) = started_session();
    let play = ShengjiClassifiedPlay {
        cards: vec![card; 26],
        category: Category::Suit(ShengjiSuit::Diamond),
        components: vec![
            Component::Single { card, strength: 2 },
            Component::Pair {
                cards: [card; 2],
                strength: 3,
            },
            Component::Triple {
                cards: [card; 3],
                strength: 4,
            },
            Component::Tractor {
                cards: vec![card; 6],
                pair_count: 3,
                top_strength: 8,
            },
            Component::Titanic {
                cards: vec![card; 6],
                triple_count: 2,
                top_strength: 9,
            },
            Component::Spaceship {
                cards: vec![card; 8],
                quad_count: 2,
                top_strength: 10,
            },
        ],
    };

    session.record_profile_play(ShengjiPlayerId(0), &play, true, true);

    let stats = &session.statistics.profiles[0];
    assert_eq!(stats.plays, 1);
    assert_eq!(stats.winning_plays, 1);
    assert_eq!(stats.play_category_counts[0], 1);
    assert_eq!(stats.play_category_counts[1], 1);
    assert_eq!(stats.play_category_counts[3], 1);
    assert_eq!(stats.play_category_counts[4], 1);
    assert_eq!(stats.play_category_counts.iter().sum::<u32>(), 4);
    assert_eq!(stats.longest_tractor, 3);
    assert_eq!(stats.longest_titanic, 2);
    assert_eq!(stats.longest_space_fortress, 2);
    assert_eq!(stats.longest_throw, 26);
}

#[cfg(feature = "developer")]
#[test]
fn developer_bots_bid_bury_and_play_with_the_greedy_policy() {
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let mut deck = build_deck();
    let target_index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(1, target_index);
    let mut session = ShengjiSession::new(ROOM, ShengjiRuleSet::default(), deck).unwrap();
    let host = ConnectionId(10);
    session.handle(host, message(0, 1, join_command(0)));
    session.handle(
        host,
        message(0, 2, ClientCommand::SelectSeat { seat: SeatId(0) }),
    );
    for seat in 1..PLAYER_COUNT {
        session.handle(
            host,
            message(
                0,
                u64::from(seat) + 2,
                ClientCommand::ConfigureBotSeat {
                    seat: SeatId(seat),
                    occupied: true,
                },
            ),
        );
    }
    session.handle(host, message(0, 6, ClientCommand::StartGame));
    let deliveries = session.advance_time(DEAL_INTERVAL * 100);
    let bidding = game_snapshot(&deliveries, host);
    assert_eq!(bidding.declaration.unwrap().player, PlayerId(1));

    session.advance_time(BIDDING_GRACE);
    session.advance_time(AUTOMATIC_ACTION_DELAY);
    let deliveries = session.advance_time(AUTOMATIC_ACTION_DELAY);
    let playing = game_snapshot(&deliveries, host);
    assert!(matches!(playing.phase, ShengjiPhaseView::Playing));
    assert_eq!(
        playing.trick.as_ref().map(|trick| trick.plays.len()),
        Some(1)
    );
    assert_eq!(playing.current_player, Some(PlayerId(2)));
}
