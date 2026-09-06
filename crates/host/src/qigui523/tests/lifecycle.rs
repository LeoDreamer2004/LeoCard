use super::*;
use leocard_protocol::{
    AVATAR_DIMENSION, ClientCommand, PlayerId, ReconnectToken, RejectReason, ServerEvent,
};
use leocard_qigui523::{Phase, build_deck};

#[test]
fn player_still_in_auto_play_is_ready_and_remains_in_auto_play_next_game() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    session
        .players
        .iter_mut()
        .find(|player| player.connection == SECOND)
        .unwrap()
        .auto_play = true;

    finish_game(&mut session);
    session.broadcast_game_after_update(None);

    let auto_player = session
        .players
        .iter()
        .find(|player| player.connection == SECOND)
        .unwrap();
    assert!(auto_player.ready);
    assert!(auto_player.auto_play);
    assert!(
        session
            .players
            .iter()
            .filter(|player| player.connection != SECOND)
            .all(|player| !player.ready)
    );

    send_next(&mut session, HOST, ClientCommand::PlayAgain);
    send_next(&mut session, THIRD, ClientCommand::PlayAgain);

    assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
    let auto_player = session
        .players
        .iter()
        .find(|player| player.connection == SECOND)
        .unwrap();
    assert!(!auto_player.ready);
    assert!(auto_player.auto_play);
}

#[test]
fn guest_leaves_with_a_named_notice_and_no_longer_blocks_the_next_game() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    finish_game(&mut session);

    let deliveries = send_next(&mut session, SECOND, ClientCommand::LeaveRoom);
    assert!(deliveries.iter().any(|delivery| {
        delivery.recipient == SECOND && matches!(delivery.message.event, ServerEvent::LeftRoom)
    }));
    assert_eq!(
        deliveries
            .iter()
            .filter(|delivery| matches!(
                &delivery.message.event,
                ServerEvent::PlayerLeft { name } if name == "玩家二"
            ))
            .count(),
        2
    );

    send_next(&mut session, HOST, ClientCommand::PlayAgain);
    let next_game = send_next(&mut session, THIRD, ClientCommand::PlayAgain);
    assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
    assert_eq!(session.game().unwrap().players().len(), 2);
    assert_eq!(next_game.len(), 2);
    assert!(session.players.iter().all(|player| player.name != "玩家二"));
}

#[test]
fn player_still_offline_at_game_end_is_automatically_departed() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    session.disconnect(SECOND);
    assert!(
        !session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .unwrap()
            .left
    );

    finish_game(&mut session);
    let deliveries = session.broadcast_game_after_update(None);

    let departed = session
        .players
        .iter()
        .find(|player| player.connection == SECOND)
        .unwrap();
    assert!(departed.left);
    assert!(!departed.ready);
    assert_eq!(
        deliveries
            .iter()
            .filter(|delivery| matches!(
                &delivery.message.event,
                ServerEvent::PlayerLeft { name } if name == "玩家二"
            ))
            .count(),
        2
    );

    send_next(&mut session, HOST, ClientCommand::PlayAgain);
    send_next(&mut session, THIRD, ClientCommand::PlayAgain);
    assert_eq!(session.game().unwrap().players().len(), 2);
    assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
}

#[test]
fn host_disconnect_closes_the_room_immediately() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let deliveries = session.disconnect(HOST);

    assert!(session.is_closed());
    assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
    assert_eq!(deliveries.len(), 2);
    assert!(
        deliveries
            .iter()
            .all(|delivery| matches!(delivery.message.event, ServerEvent::RoomClosed))
    );
}

#[test]
fn host_leave_command_during_a_game_closes_the_room_for_everyone() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let deliveries = send_next(&mut session, HOST, ClientCommand::LeaveRoom);

    assert!(session.is_closed());
    assert_eq!(deliveries.len(), 3);
    assert!(
        deliveries
            .iter()
            .all(|delivery| matches!(delivery.message.event, ServerEvent::RoomClosed))
    );
}

#[test]
fn lobby_leave_slot_is_reused_without_renumbering_the_remaining_players() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    send_next(&mut session, SECOND, ClientCommand::LeaveRoom);

    let fourth = ConnectionId(40);
    let deliveries = session.handle(
        fourth,
        message(1, join_command("新玩家", ReconnectToken(fourth.0))),
    );

    assert_eq!(session.player_id(THIRD), Some(PlayerId(2)));
    assert_eq!(session.player_id(fourth), Some(PlayerId(1)));
    assert!(deliveries.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::Joined { you: PlayerId(1) }
    )));
    assert!(
        session
            .lobby_snapshot()
            .players
            .iter()
            .all(|player| player.name != "玩家二")
    );
}

#[test]
fn seventh_connection_is_rejected_from_the_six_player_room() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    for index in 0..6_u64 {
        session.handle(
            ConnectionId(100 + index),
            message(
                1,
                join_command(&format!("P{index}"), ReconnectToken(100 + index)),
            ),
        );
    }
    let seventh = session.handle(
        ConnectionId(200),
        message(1, join_command("P6", ReconnectToken(200))),
    );
    assert_eq!(rejection(&seventh), Some(&RejectReason::RoomFull));
}

#[test]
fn normalized_avatar_is_sent_once_to_each_current_or_late_joiner() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    session.handle(
        HOST,
        message(1, join_command("房主", ReconnectToken(HOST.0))),
    );
    let png = avatar_png(AVATAR_DIMENSION, AVATAR_DIMENSION);
    let upload = session.handle(
        HOST,
        message(2, ClientCommand::SetAvatar { png: png.clone() }),
    );
    assert_eq!(
        upload
            .iter()
            .filter(|delivery| {
                matches!(&delivery.message.event, ServerEvent::AvatarData { .. })
            })
            .count(),
        1
    );

    let late_join = session.handle(
        SECOND,
        message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
    );
    let avatars: Vec<_> = late_join
        .iter()
        .filter(|delivery| delivery.recipient == SECOND)
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::AvatarData { id, png } => Some((*id, png)),
            _ => None,
        })
        .collect();
    assert_eq!(avatars.len(), 1);
    assert_eq!(avatars[0].1, &png);

    let duplicate = session.handle(
        HOST,
        message(3, ClientCommand::SetAvatar { png: png.clone() }),
    );
    assert_eq!(rejection(&duplicate), Some(&RejectReason::AvatarAlreadySet));
    let invalid = session.handle(
        SECOND,
        message(
            2,
            ClientCommand::SetAvatar {
                png: avatar_png(AVATAR_DIMENSION / 2, AVATAR_DIMENSION / 2),
            },
        ),
    );
    assert_eq!(rejection(&invalid), Some(&RejectReason::InvalidAvatar));
}
