use super::*;
use leocard_protocol::{
    ChatContent, ClientCommand, GameCommand, GameEvent, GameViolation, PlayerId,
    PlayerInteractionKind, QUICK_VOICE_COUNT, QiGui523Command, QiGui523Event, ReconnectToken,
    RejectReason, RequestId, RequestViolation, Revision, RoomId, RoomViolation, RuleViolation,
    SeatId, ServerEvent,
};
use leocard_qigui523::QiGuiPlayKind;
use leocard_qigui523::build_deck;

#[test]
fn each_game_snapshot_contains_only_its_recipient_hand() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    let deliveries = ready_and_start(&mut session);
    let game = session.game().unwrap();

    for delivery in deliveries {
        let ServerEvent::GameSnapshot(snapshot) = delivery.message.event else {
            panic!("start broadcasts personalized game snapshots");
        };
        let snapshot = snapshot.into_qigui523().expect("七鬼五二三快照");
        let expected_player = session.player_id(delivery.recipient).unwrap();
        assert_eq!(snapshot.you, expected_player);
        assert_eq!(
            snapshot.your_hand,
            game.players()[usize::from(expected_player.0)].hand()
        );
        assert!(
            snapshot
                .players
                .iter()
                .all(|player| player.hand_len == rules().hand_size.into())
        );
    }
}

#[test]
fn duplicate_action_request_never_executes_twice() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let current = session.game().unwrap().trick().unwrap().current_player();
    let connection = session.players[current.0].connection;
    let card = session.game().unwrap().players()[current.0].hand()[0];
    let command = message(
        10,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: vec![card],
        })),
    );
    let revision_before = session.revision();

    session.handle(connection, command.clone());
    let revision_after_first = session.revision();
    let hand_after_first = session.game().unwrap().players()[current.0].hand().len();
    let duplicate = session.handle(connection, command);

    assert_eq!(revision_after_first.0, revision_before.0 + 1);
    assert_eq!(session.revision(), revision_after_first);
    assert_eq!(
        session.game().unwrap().players()[current.0].hand().len(),
        hand_after_first
    );
    assert_eq!(
        rejection(&duplicate),
        Some(&RejectReason::Request(RequestViolation::DuplicateRequest {
            last_seen: RequestId(10)
        }))
    );
}

#[test]
fn accepted_play_broadcasts_one_effect_event_to_every_connected_player() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let starting = session.game().unwrap().starting_card();
    let player = from_core_player(starting.player);
    let connection = connection_for_player(&session, player);

    let deliveries = send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: vec![starting.card],
        })),
    );

    assert_eq!(
        deliveries
            .iter()
            .filter(|delivery| matches!(
                &delivery.message.event,
                ServerEvent::GameEvent(GameEvent::QiGui523(
                    QiGui523Event::PlayEffect {
                        player: effect_player,
                        play,
                    }
                )) if *effect_player == player
                    && play.cards == vec![starting.card]
                    && matches!(play.kind, QiGuiPlayKind::Single)
            ))
            .count(),
        3
    );
}

#[test]
fn player_interaction_is_broadcast_identically_to_every_connected_player() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let deliveries = session.handle(
        HOST,
        message(
            10,
            ClientCommand::Interact {
                target: PlayerId(2),
                kind: PlayerInteractionKind::Egg,
            },
        ),
    );
    let interactions = deliveries
        .iter()
        .filter_map(|delivery| match delivery.message.event {
            ServerEvent::PlayerInteraction(interaction) => Some((delivery, interaction)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(interactions.len(), 3);
    assert!(interactions.iter().all(|(_, interaction)| {
        interaction.source == PlayerId(0)
            && interaction.target == PlayerId(2)
            && interaction.kind == PlayerInteractionKind::Egg
            && interaction.seed == interactions[0].1.seed
    }));
    assert!(interactions.iter().all(|(delivery, _)| {
        delivery.message.in_reply_to == (delivery.recipient == HOST).then_some(RequestId(10))
    }));
    assert_eq!(
        session
            .players
            .iter()
            .find(|player| player.id == PlayerId(2))
            .and_then(|player| player.game_profiles.interactions.as_ref())
            .map(|stats| stats.eggs_received),
        Some(1)
    );
}

#[test]
fn interaction_profile_converts_wine_and_shoe_into_ten_items() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);

    session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Flower);
    session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Wine);
    session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Egg);
    session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Shoe);

    let stats = session
        .players
        .iter()
        .find(|player| player.id == PlayerId(1))
        .and_then(|player| player.game_profiles.interactions.as_ref())
        .unwrap();
    assert_eq!(stats.flowers_received, 11);
    assert_eq!(stats.eggs_received, 11);
}

#[test]
fn chat_is_validated_and_broadcast_to_every_connected_player() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let deliveries = session.handle(
        HOST,
        message(
            10,
            ClientCommand::Chat {
                content: ChatContent::Text("  大家好  ".to_owned()),
            },
        ),
    );
    let chats = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::ChatMessage(chat) => Some((delivery, chat)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(chats.len(), 3);
    assert!(chats.iter().all(|(_, chat)| {
        chat.source == PlayerId(0) && chat.content == ChatContent::Text("大家好".to_owned())
    }));
    assert!(chats.iter().all(|(delivery, _)| {
        delivery.message.in_reply_to == (delivery.recipient == HOST).then_some(RequestId(10))
    }));

    let invalid = session.handle(
        HOST,
        message(
            11,
            ClientCommand::Chat {
                content: ChatContent::QuickVoice(QUICK_VOICE_COUNT),
            },
        ),
    );
    assert_eq!(
        rejection(&invalid),
        Some(&RejectReason::Room(RoomViolation::InvalidChatMessage))
    );
}

#[test]
fn a_player_cannot_send_an_interaction_to_themselves() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let deliveries = session.handle(
        SECOND,
        message(
            10,
            ClientCommand::Interact {
                target: PlayerId(1),
                kind: PlayerInteractionKind::Flower,
            },
        ),
    );

    assert_eq!(
        rejection(&deliveries),
        Some(&RejectReason::Game(GameViolation::QiGui523(
            RuleViolation::InvalidPlayer,
        )))
    );
}

#[test]
fn rejected_rule_action_does_not_mutate_authoritative_state() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let current = session.game().unwrap().trick().unwrap().current_player();
    let wrong_index = (current.0 + 1) % 3;
    let wrong_connection = session.players[wrong_index].connection;
    let before = session.game().unwrap().clone();
    let revision = session.revision();
    let deliveries = session.handle(
        wrong_connection,
        message(
            10,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::Pass)),
        ),
    );

    assert_eq!(session.game(), Some(&before));
    assert_eq!(session.revision(), revision);
    assert_eq!(
        rejection(&deliveries),
        Some(&RejectReason::Game(GameViolation::QiGui523(
            RuleViolation::NotPlayersTurn,
        )))
    );
}

#[test]
fn protocol_and_room_mismatch_never_touch_room_state() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    let mut wrong_protocol = message(1, join_command("玩家", ReconnectToken(HOST.0)));
    wrong_protocol.protocol_version += 1;
    let protocol_reply = session.handle(HOST, wrong_protocol);

    let mut wrong_room = message(1, join_command("玩家", ReconnectToken(HOST.0)));
    wrong_room.room_id = RoomId(999);
    let room_reply = session.handle(HOST, wrong_room);

    assert!(matches!(
        rejection(&protocol_reply),
        Some(RejectReason::Request(
            RequestViolation::ProtocolMismatch { .. }
        ))
    ));
    assert_eq!(
        rejection(&room_reply),
        Some(&RejectReason::Request(RequestViolation::RoomMismatch))
    );
    assert!(session.players.is_empty());
    assert_eq!(session.revision(), Revision(0));
}

#[test]
fn disconnect_is_broadcast_once_only_to_connected_players() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    let revision = session.revision();

    let deliveries = session.disconnect(SECOND);

    assert_eq!(session.revision().0, revision.0 + 1);
    assert_eq!(deliveries.len(), 2);
    assert!(
        deliveries
            .iter()
            .all(|delivery| delivery.recipient != SECOND)
    );
    for delivery in deliveries {
        let ServerEvent::LobbySnapshot(snapshot) = delivery.message.event else {
            panic!("disconnect before a game broadcasts lobby snapshots");
        };
        assert_eq!(snapshot.players.len(), 2);
        assert!(snapshot.players.iter().all(|player| player.connected));
        assert!(
            snapshot
                .players
                .iter()
                .all(|player| player.id != PlayerId(1))
        );
    }

    assert!(session.players[usize::from(PlayerId(1).0)].left);

    let freed_seat = session.handle(
        HOST,
        message(4, ClientCommand::SelectSeat { seat: SeatId(1) }),
    );
    assert_eq!(rejection(&freed_seat), None);
    assert_eq!(
        session
            .players
            .iter()
            .find(|player| player.connection == HOST)
            .unwrap()
            .seat,
        Some(SeatId(1))
    );

    let revision_after_reseat = session.revision();
    assert!(session.disconnect(SECOND).is_empty());
    assert_eq!(session.revision(), revision_after_reseat);

    let replacement = ConnectionId(99);
    let rejoined = session.handle(
        replacement,
        message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
    );
    assert_eq!(session.player_id(replacement), Some(PlayerId(1)));
    assert!(!session.players[usize::from(PlayerId(1).0)].left);
    assert!(rejoined.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::LobbySnapshot(snapshot) if snapshot.players.len() == 3
    )));
}

#[test]
fn reconnect_during_game_restores_the_same_player_and_private_hand() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let player = session.player_id(SECOND).unwrap();
    let hand_before = session.game_snapshot(player).your_hand;

    session.disconnect(SECOND);
    let replacement = ConnectionId(99);
    let deliveries = session.handle(
        replacement,
        message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
    );

    assert_eq!(session.player_id(SECOND), None);
    assert_eq!(session.player_id(replacement), Some(player));
    assert!(session.players[usize::from(player.0)].connected);
    assert!(deliveries.iter().any(|delivery| {
        delivery.recipient == replacement
            && matches!(
                delivery.message.event,
                ServerEvent::Joined { you } if you == player
            )
    }));
    let restored_hand = deliveries.iter().find_map(|delivery| {
        if delivery.recipient != replacement {
            return None;
        }
        let ServerEvent::GameSnapshot(snapshot) = &delivery.message.event else {
            return None;
        };
        Some(snapshot.qigui523()?.your_hand.clone())
    });
    assert_eq!(restored_hand, Some(hand_before));
}
