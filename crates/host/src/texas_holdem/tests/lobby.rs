use super::*;

#[test]
fn lobby_start_waits_for_both_blind_actions() {
    let mut session = session_waiting_for_blinds();
    let first = session
        .game()
        .unwrap()
        .snapshot(session.room.players[0].id)
        .unwrap()
        .blind_to_post
        .unwrap();
    let connection = connection_for(&session, first.player);
    let delivered = session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::PostBlind,
            })),
        ),
    );
    assert!(delivered.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(TexasHoldemEvent::ActionApplied {
            action: TexasHoldemAction::PostBlind,
            ..
        }))
    )));
    assert!(session.game().unwrap().game().blind_to_post().is_some());
}

#[test]
fn texas_auto_play_waits_one_second_then_uses_the_passive_bot_action() {
    let mut session = session_waiting_for_blinds();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay {
                enabled: true,
            })),
        ),
    );

    let snapshot = session.game_snapshot(current);
    assert!(
        snapshot
            .players
            .iter()
            .find(|player| player.id == current)
            .unwrap()
            .auto_play
    );
    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    let deliveries = session.advance_time(Duration::from_millis(1));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(
            TexasHoldemEvent::ActionApplied {
                player,
                action: TexasHoldemAction::PostBlind,
                ..
            }
        )) if player == current
    )));
}

#[cfg(feature = "developer")]
#[test]
fn developer_host_right_click_commands_add_default_texas_bots() {
    let mut session = TexasHoldemSession::new_with_host_port(
        ROOM,
        52301,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
    let rejected = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::NotEnoughPlayers { .. })
        }
    )));
    let empty_seats = (0..TABLE_SEAT_COUNT)
        .map(SeatId)
        .filter(|seat| {
            session
                .room
                .players
                .iter()
                .all(|player| player.seat != Some(*seat))
        })
        .take(2)
        .collect::<Vec<_>>();
    for (index, seat) in empty_seats.into_iter().enumerate() {
        session.handle(
            HOST_CONNECTION,
            message(
                3 + index as u64,
                ClientCommand::ConfigureBotSeat {
                    seat,
                    occupied: true,
                },
            ),
        );
    }
    let deliveries = session.handle(HOST_CONNECTION, message(5, ClientCommand::StartGame));

    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
    )));
    assert_eq!(
        session.room.players.len(),
        TexasHoldemRuleSet::MIN_PLAYERS as usize
    );
    assert_eq!(
        session
            .room
            .players
            .iter()
            .filter(|player| player.is_bot)
            .count(),
        TexasHoldemRuleSet::MIN_PLAYERS as usize - 1
    );
    for bot in session.room.players.iter().filter(|player| player.is_bot) {
        assert_eq!(bot.profile_id, ProfileId([0; 32]));
        assert_eq!(bot.avatar, None);
        assert_eq!(bot.reference_points, 0);
        assert_eq!(bot.completed_games, 0);
        assert!(bot.auto_play);
    }
    let snapshot = session.game_snapshot(session.room.host_player_id().unwrap());
    assert!(
        snapshot
            .players
            .iter()
            .filter(|player| player.auto_play)
            .count()
            >= 2
    );
}

#[test]
fn common_lobby_starts_texas_and_each_snapshot_has_only_its_own_hole_cards() {
    let session = started_session();
    assert_eq!(session.rules().player_count, TABLE_SEAT_COUNT);
    let game = session.game().unwrap();
    let snapshots = session
        .room
        .players
        .iter()
        .map(|player| game.snapshot(player.id).unwrap())
        .collect::<Vec<_>>();
    assert!(snapshots.iter().all(|snapshot| {
        snapshot.your_hole_cards.len() == 2 && snapshot.revealed_hands.is_empty()
    }));
    assert_ne!(snapshots[0].your_hole_cards, snapshots[1].your_hole_cards);
    assert!(snapshots.iter().all(|snapshot| snapshot.host_port == 52301));
    assert_eq!(snapshots[0].players.len(), 3);
    assert_eq!(snapshots[0].pot, 3);
}

#[test]
fn protocol_actions_use_the_adapter_and_rejections_leave_state_unchanged() {
    let mut session = started_session();
    let current = session.game().unwrap().current_player().unwrap();
    let connection = connection_for(&session, current);
    let before = session.game().unwrap().game().clone();
    let rejected = session.handle(
        connection,
        message(
            3,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::Check,
            })),
        ),
    );
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::TexasHoldem(
                TexasHoldemViolation::CannotCheckWhileFacingBet { .. }
            ))
        }
    )));
    assert_eq!(session.game().unwrap().game(), &before);

    let accepted = session.handle(
        connection,
        message(
            4,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::Call,
            })),
        ),
    );
    assert!(accepted.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::TexasHoldem(
            TexasHoldemEvent::ActionApplied {
                player,
                action: TexasHoldemAction::Call,
                ..
            }
        )) if player == current
    )));
}

#[test]
fn texas_rules_are_visible_in_the_shared_lobby_snapshot() {
    let mut session = TexasHoldemSession::new(
        ROOM,
        TexasHoldemRuleSet {
            short_deck: true,
            ignore_kickers: true,
            omaha: true,
            ..TexasHoldemRuleSet::default()
        },
        build_deck(true),
    )
    .unwrap();
    let deliveries = session.handle(HOST_CONNECTION, message(1, join_command("房主", 11)));
    let lobby = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::LobbySnapshot(snapshot) => Some(snapshot),
            _ => None,
        })
        .unwrap();
    assert_eq!(lobby.game, GameKind::TexasHoldem);
    assert!(lobby.rules.texas_holdem().unwrap().short_deck);
    assert!(lobby.rules.texas_holdem().unwrap().ignore_kickers);
    assert!(lobby.rules.texas_holdem().unwrap().omaha);
    assert_eq!(lobby.host_port, 52300);
    assert!(session.game().is_none());
}

#[test]
fn lobby_disconnect_releases_the_player_from_count_and_capacity() {
    let mut session =
        TexasHoldemSession::new(ROOM, TexasHoldemRuleSet::default(), build_deck(false)).unwrap();
    session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
    session.handle(SECOND_CONNECTION, message(1, join_command("玩家二", 2)));
    session.handle(THIRD_CONNECTION, message(1, join_command("玩家三", 3)));

    let deliveries = session.disconnect(SECOND_CONNECTION);

    assert!(session.room.players[1].left);
    assert_eq!(
        session
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count(),
        2
    );
    assert!(deliveries.iter().all(|delivery| {
        let ServerEvent::LobbySnapshot(snapshot) = &delivery.message.event else {
            return false;
        };
        snapshot.players.len() == 2 && snapshot.players.iter().all(|player| player.connected)
    }));

    let replacement = ConnectionId(40);
    let rejoined = session.handle(replacement, message(1, join_command("新玩家", 4)));
    assert!(rejoined.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::LobbySnapshot(snapshot) if snapshot.players.len() == 3
    )));
}
