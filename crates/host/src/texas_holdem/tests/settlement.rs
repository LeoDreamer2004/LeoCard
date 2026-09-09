use super::*;

#[test]
fn all_players_ready_start_the_next_hand_with_stacks_and_rotated_dealer_preserved() {
    let mut session = started_session();
    let first_dealer = session.game().unwrap().game().dealer();
    let mut requests = HashMap::from([
        (HOST_CONNECTION, 2_u64),
        (SECOND_CONNECTION, 2_u64),
        (THIRD_CONNECTION, 2_u64),
    ]);
    while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        let request = requests.get_mut(&connection).unwrap();
        *request += 1;
        session.handle(
            connection,
            message(
                *request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::Fold,
                })),
            ),
        );
    }
    assert!(matches!(
        session.game().unwrap().game().phase(),
        Phase::Complete(_)
    ));
    for (index, connection) in [HOST_CONNECTION, SECOND_CONNECTION, THIRD_CONNECTION]
        .into_iter()
        .enumerate()
    {
        let request = requests.get_mut(&connection).unwrap();
        *request += 1;
        session.handle(connection, message(*request, ClientCommand::PlayAgain));
        if index < 2 {
            let snapshot = session.game_snapshot(session.room.players[0].id);
            assert!(matches!(
                snapshot.phase,
                TexasHoldemPhaseView::HandComplete { .. }
            ));
            assert_eq!(
                snapshot
                    .players
                    .iter()
                    .filter(|player| player.ready)
                    .count(),
                index + 1
            );
        }
    }
    let game = session.game().unwrap().game();
    assert_eq!(game.hand_number(), 1);
    assert_ne!(game.dealer(), first_dealer);
    assert!(matches!(game.phase(), Phase::Betting(_)));
}

#[test]
fn first_busted_player_finishes_the_tournament_and_applies_shared_rating_once() {
    let prefix = [
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::King),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Queen),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::King),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Queen),
        TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Ace),
        TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Two),
        TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Three),
        TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Seven),
        TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Eight),
        TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Nine),
    ];
    let deck = prefix
        .into_iter()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !prefix.contains(card)),
        )
        .collect();
    let mut session = TexasHoldemSession::new_with_host_port(
        ROOM,
        52301,
        TexasHoldemRuleSet {
            starting_chips: 5,
            ..TexasHoldemRuleSet::default()
        },
        deck,
    )
    .unwrap();
    for (connection, name, token) in [
        (HOST_CONNECTION, "房主", 1),
        (SECOND_CONNECTION, "玩家二", 2),
        (THIRD_CONNECTION, "玩家三", 3),
    ] {
        session.handle(connection, message(1, join_command(name, token)));
    }
    for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    let mut request = 3;
    while session.game().unwrap().game().blind_to_post().is_some() {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::PostBlind,
                })),
            ),
        );
        request += 1;
    }
    while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        session.handle(
            connection,
            message(
                request,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::AllIn,
                })),
            ),
        );
        request += 1;
    }
    let snapshot = session.game_snapshot(session.room.players[0].id);
    let TexasHoldemPhaseView::HandComplete {
        tournament_complete,
        reference_changes,
        ..
    } = snapshot.phase
    else {
        panic!("all-in showdown should complete the hand");
    };
    assert!(
        tournament_complete,
        "final stacks: {:?}",
        snapshot
            .players
            .iter()
            .map(|player| player.stack)
            .collect::<Vec<_>>()
    );
    assert_eq!(reference_changes.len(), 3);
    assert!(snapshot.players.iter().any(|player| player.stack == 0));
    for player in &snapshot.players {
        let stats = player.game_profiles.texas_holdem.as_ref().unwrap();
        let delta = reference_changes
            .iter()
            .find(|change| change.player == player.id)
            .unwrap()
            .delta;
        let placement = 1 + snapshot
            .players
            .iter()
            .filter(|other| other.stack > player.stack)
            .count();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(stats.total_reference_delta, i64::from(delta));
        assert_eq!(stats.total_final_chips, u64::from(player.stack));
        assert_eq!(stats.placement_counts[placement - 1], 1);
        assert_eq!(stats.hands_played, 1);
        assert_eq!(stats.all_in_actions, 1);
        assert_eq!(stats.wager_actions, 1);
        assert_eq!(stats.hand_category_counts.iter().sum::<u32>(), 1);
    }
    let points_after = session
        .room
        .players
        .iter()
        .map(|player| player.reference_points)
        .collect::<Vec<_>>();
    session.apply_finished_reference_points();
    assert_eq!(
        session
            .room
            .players
            .iter()
            .map(|player| player.reference_points)
            .collect::<Vec<_>>(),
        points_after
    );
}

#[test]
fn texas_profile_action_statistics_ignore_blinds_and_zero_value_actions() {
    let mut session =
        TexasHoldemSession::new(ROOM, TexasHoldemRuleSet::default(), build_deck(false)).unwrap();
    session.match_profile_stats = vec![TexasHoldemProfileStats::default(); 2];
    session.record_profile_events(&[
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::PostBlind,
            amount: 2,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::Check,
            amount: 0,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::RaiseTo(12),
            amount: 10,
        },
        TexasHoldemEvent::ActionApplied {
            player: PlayerId(0),
            action: TexasHoldemAction::Fold,
            amount: 0,
        },
    ]);

    let stats = &session.match_profile_stats[0];
    assert_eq!(stats.voluntary_actions, 3);
    assert_eq!(stats.check_actions, 1);
    assert_eq!(stats.raise_actions, 1);
    assert_eq!(stats.hands_folded, 1);
    assert_eq!(stats.wager_actions, 1);
    assert_eq!(stats.wagered_chips, 10);
}

#[test]
fn disconnecting_the_current_guest_immediately_folds_them() {
    let mut session = started_session();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    if connection == HOST_CONNECTION {
        // 先合法行动一次，确保测试目标是可断线而不关闭房间的客人。
        session.handle(
            connection,
            message(
                3,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: TexasHoldemAction::Call,
                })),
            ),
        );
    }
    let guest = session.game().unwrap().current_player().unwrap();
    let guest_connection = connection_for(&session, guest);
    assert_ne!(guest_connection, HOST_CONNECTION);
    let deliveries = session.disconnect(guest_connection);
    assert!(!deliveries.is_empty());
    let core_index = session
        .game()
        .unwrap()
        .players()
        .iter()
        .position(|player| player.id == guest)
        .unwrap();
    assert!(session.game().unwrap().game().players()[core_index].folded());
}
