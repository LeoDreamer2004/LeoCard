use super::*;
use leocard_qigui523::{SameCardPolicy, SuitComparison};
#[test]
fn only_ready_seated_room_host_can_start() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);

    let non_host = session.handle(SECOND, message(3, ClientCommand::StartGame));
    assert_eq!(rejection(&non_host), Some(&RejectReason::OnlyHostCanStart));

    let not_ready = session.handle(HOST, message(3, ClientCommand::StartGame));
    assert!(matches!(
        rejection(&not_ready),
        Some(RejectReason::PlayersNotReady { .. })
    ));

    for connection in [HOST, SECOND, THIRD] {
        session.handle(
            connection,
            message(4, ClientCommand::SetReady { ready: true }),
        );
    }
    let deliveries = session.handle(HOST, message(5, ClientCommand::StartGame));
    assert!(session.game().is_some());
    assert_eq!(deliveries.len(), 3);
}

#[test]
fn only_host_can_update_rules_and_changes_reset_ready_state() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    for connection in [HOST, SECOND, THIRD] {
        session.handle(
            connection,
            message(3, ClientCommand::SetReady { ready: true }),
        );
    }
    let seats_before: HashMap<_, _> = session
        .players
        .iter()
        .map(|player| (player.connection, player.seat))
        .collect();
    let configured = QiGuiRuleSet {
        deck_count: 2,
        hand_size: 10,
        suit_comparison: SuitComparison::SumPoints,
        same_card_policy: SameCardPolicy::CanFollow,
        ..rules()
    };

    let non_host = session.handle(
        SECOND,
        message(
            4,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                rules: configured,
            })),
        ),
    );
    assert_eq!(
        rejection(&non_host),
        Some(&RejectReason::OnlyHostCanConfigure)
    );
    let invalid = session.handle(
        HOST,
        message(
            4,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                rules: QiGuiRuleSet {
                    deck_count: 0,
                    ..configured
                },
            })),
        ),
    );
    assert_eq!(
        rejection(&invalid),
        Some(&RejectReason::InvalidRuleConfiguration)
    );

    let deliveries = session.handle(
        HOST,
        message(
            5,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                rules: configured,
            })),
        ),
    );
    assert_eq!(*session.rules(), configured);
    assert_eq!(session.shuffled_deck.as_ref().unwrap().len(), 108);
    assert!(
        session
            .players
            .iter()
            .all(|player| { player.ready == (player.connection == HOST) })
    );
    assert_eq!(
        session
            .players
            .iter()
            .map(|player| (player.connection, player.seat))
            .collect::<HashMap<_, _>>(),
        seats_before
    );
    assert_eq!(deliveries.len(), 3);
    assert!(deliveries.iter().all(|delivery| {
        matches!(
            &delivery.message.event,
            ServerEvent::LobbySnapshot(snapshot)
                if snapshot.qigui523_rules() == Some(&configured)
                    && snapshot.players.iter().all(|player| {
                        player.ready == (Some(player.id) == snapshot.host)
                    })
        )
    }));
}

#[test]
fn joining_assigns_unique_seats_and_the_host_is_always_ready() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    for (connection, name) in [(HOST, "房主"), (SECOND, "玩家二")] {
        session.handle(
            connection,
            message(1, join_command(name, ReconnectToken(connection.0))),
        );
    }

    let host_seat = session
        .players
        .iter()
        .find(|player| player.connection == HOST)
        .and_then(|player| player.seat)
        .unwrap();
    let second_seat = session
        .players
        .iter()
        .find(|player| player.connection == SECOND)
        .and_then(|player| player.seat)
        .unwrap();
    assert_ne!(host_seat, second_seat);
    assert!(
        session
            .players
            .iter()
            .find(|player| player.connection == HOST)
            .unwrap()
            .ready
    );
    assert!(
        !session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .unwrap()
            .ready
    );

    session.handle(HOST, message(2, ClientCommand::SetReady { ready: false }));
    assert!(
        session
            .players
            .iter()
            .find(|player| player.connection == HOST)
            .unwrap()
            .ready
    );
    let occupied = session.handle(
        SECOND,
        message(2, ClientCommand::SelectSeat { seat: host_seat }),
    );
    assert_eq!(rejection(&occupied), Some(&RejectReason::SeatTaken));
    session.handle(SECOND, message(3, ClientCommand::SetReady { ready: true }));
    let deliveries = session.handle(HOST, message(4, ClientCommand::StartGame));

    assert_eq!(session.game().unwrap().rules().player_count, 2);
    assert_eq!(deliveries.len(), 2);
    let seats: Vec<_> = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523().and_then(|snapshot| {
                snapshot
                    .players
                    .iter()
                    .find(|player| player.id == snapshot.you)
                    .map(|player| player.seat)
            }),
            _ => None,
        })
        .collect();
    assert!(seats.contains(&host_seat));
    assert!(seats.contains(&second_seat));
}

#[cfg(feature = "developer")]
#[test]
fn developer_bot_seat_changes_keep_the_host_identity_stable() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    session.handle(
        HOST,
        message(1, join_command("房主", ReconnectToken(HOST.0))),
    );
    session.handle(
        HOST,
        message(2, ClientCommand::SelectSeat { seat: SeatId(5) }),
    );
    let rejected = session.handle(HOST, message(3, ClientCommand::StartGame));
    assert!(matches!(
        rejection(&rejected),
        Some(RejectReason::NotEnoughPlayers { .. })
    ));
    let first_bot_seat = SeatId(0);
    let second_bot_seat = SeatId(1);
    session.handle(
        HOST,
        message(
            4,
            ClientCommand::ConfigureBotSeat {
                seat: first_bot_seat,
                occupied: true,
            },
        ),
    );
    session.handle(
        HOST,
        message(
            5,
            ClientCommand::ConfigureBotSeat {
                seat: second_bot_seat,
                occupied: true,
            },
        ),
    );
    assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
    assert_eq!(session.lobby_snapshot().host, Some(PlayerId(0)));

    session.handle(
        HOST,
        message(
            6,
            ClientCommand::ConfigureBotSeat {
                seat: first_bot_seat,
                occupied: false,
            },
        ),
    );
    assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
    assert_eq!(
        session.players.iter().filter(|player| !player.left).count(),
        2
    );
    session.handle(
        HOST,
        message(
            7,
            ClientCommand::ConfigureBotSeat {
                seat: second_bot_seat,
                occupied: false,
            },
        ),
    );
    assert_eq!(
        session.players.iter().filter(|player| !player.left).count(),
        1
    );
    session.handle(
        HOST,
        message(
            8,
            ClientCommand::ConfigureBotSeat {
                seat: first_bot_seat,
                occupied: true,
            },
        ),
    );
    assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
    let deliveries = session.handle(HOST, message(9, ClientCommand::StartGame));

    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::QiGui523(_))
    )));
    assert_eq!(session.players.len(), 2);
    let bot = session.players.iter().find(|player| player.is_bot).unwrap();
    assert_eq!(bot.profile_id, ProfileId([0; 32]));
    assert_eq!(bot.avatar, None);
    assert_eq!(bot.reference_points, 0);
    assert_eq!(bot.completed_games, 0);
    assert!(bot.auto_play);
    let snapshot = session.game_snapshot(session.room.host_player_id().unwrap());
    let bot_state = snapshot
        .players
        .iter()
        .find(|player| player.id == bot.id)
        .unwrap();
    assert!(bot_state.connected);
    assert!(bot_state.auto_play);
}

#[test]
fn finished_game_returns_to_lobby_with_seats_preserved_and_ready_reset() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    for (connection, name, seat) in [
        (HOST, "房主", SeatId(4)),
        (SECOND, "玩家二", SeatId(1)),
        (THIRD, "玩家三", SeatId(5)),
    ] {
        session.handle(
            connection,
            message(1, join_command(name, ReconnectToken(connection.0))),
        );
        session.handle(connection, message(2, ClientCommand::SelectSeat { seat }));
        session.handle(
            connection,
            message(3, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST, message(4, ClientCommand::StartGame));

    let host_player = session.player_id(HOST).unwrap();
    assert_ne!(host_player, PlayerId(0));
    assert_eq!(session.game_snapshot(host_player).host, host_player);
    let during_game = session.handle(HOST, message(5, ClientCommand::ReturnToLobby));
    assert_eq!(
        rejection(&during_game),
        Some(&RejectReason::GameNotFinished)
    );

    finish_game(&mut session);
    let non_host = session.handle(SECOND, message(5, ClientCommand::ReturnToLobby));
    assert_eq!(
        rejection(&non_host),
        Some(&RejectReason::OnlyHostCanReturnToLobby)
    );
    let seats_before: HashMap<_, _> = session
        .players
        .iter()
        .map(|player| (player.connection, player.seat))
        .collect();
    let deliveries = session.handle(HOST, message(6, ClientCommand::ReturnToLobby));

    assert!(session.game().is_none());
    assert!(session.shuffled_deck.is_some());
    assert!(
        session
            .players
            .iter()
            .all(|player| { player.ready == (player.connection == HOST) })
    );
    assert_eq!(
        session
            .players
            .iter()
            .map(|player| (player.connection, player.seat))
            .collect::<HashMap<_, _>>(),
        seats_before
    );
    assert_eq!(deliveries.len(), 3);
    for delivery in deliveries {
        let ServerEvent::LobbySnapshot(snapshot) = delivery.message.event else {
            panic!("returning to the lobby broadcasts lobby snapshots");
        };
        assert_eq!(snapshot.host, Some(host_player));
        assert!(
            snapshot
                .players
                .iter()
                .all(|player| { player.ready == (Some(player.id) == snapshot.host) })
        );
    }

    for connection in [HOST, SECOND, THIRD] {
        session.handle(
            connection,
            message(7, ClientCommand::SetReady { ready: true }),
        );
    }
    let second_game = session.handle(HOST, message(8, ClientCommand::StartGame));
    assert!(session.game().is_some());
    assert_eq!(second_game.len(), 3);
}

#[test]
fn returning_to_lobby_excludes_players_who_disconnected_after_game_end() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    finish_game(&mut session);
    session.disconnect(SECOND);

    let deliveries = send_next(&mut session, HOST, ClientCommand::ReturnToLobby);

    assert_eq!(deliveries.len(), 2);
    assert!(deliveries.iter().all(|delivery| {
        matches!(
            &delivery.message.event,
            ServerEvent::LobbySnapshot(snapshot)
                if snapshot.players.iter().all(|player| player.name != "玩家二")
        )
    }));
}

#[test]
fn finished_players_ready_in_place_and_the_last_one_starts_the_next_game() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    finish_game(&mut session);
    assert!(session.players.iter().all(|player| !player.ready));

    for connection in [HOST, SECOND] {
        let deliveries = send_next(&mut session, connection, ClientCommand::PlayAgain);
        assert!(matches!(
            session.game().unwrap().phase(),
            Phase::Finished(_)
        ));
        assert!(deliveries.iter().all(|delivery| {
            qigui523_snapshot(&delivery.message.event)
                .is_some_and(|snapshot| snapshot.players.iter().any(|player| player.ready))
        }));
    }

    let deliveries = send_next(&mut session, THIRD, ClientCommand::PlayAgain);
    assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
    assert_eq!(session.game().unwrap().players().len(), 3);
    assert_eq!(deliveries.len(), 3);
    assert!(deliveries.iter().all(|delivery| {
        qigui523_snapshot(&delivery.message.event).is_some_and(|snapshot| {
            matches!(snapshot.phase, GamePhaseView::Playing)
                && snapshot.players.iter().all(|player| !player.ready)
        })
    }));
}
