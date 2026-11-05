use super::MahjongLinkSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_mahjong_link::{
    LinkCell, MahjongLinkAction, MahjongLinkError, MahjongLinkRuleSet, PushSide, build_deck,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameSnapshot, GameViolation, JoinRequest, MahjongLinkCommand,
    MahjongLinkViolation, PlayerGameProfiles, ProfileId, ReconnectToken, RejectReason, RequestId,
    RoomId, RoomViolation, SeatId, ServerEvent,
};
use std::time::Duration;

fn send(
    session: &mut MahjongLinkSession,
    connection: u64,
    command: ClientCommand,
) -> Vec<Delivery> {
    let connection = ConnectionId(connection);
    let request = session
        .room
        .last_requests
        .get(&connection)
        .map_or(1, |request| request.0 + 1);
    session.handle(
        connection,
        ClientMessage::new(RoomId(99), RequestId(request), command),
    )
}

fn join(session: &mut MahjongLinkSession, connection: u64) {
    join_as(session, connection, connection);
}

fn join_as(session: &mut MahjongLinkSession, connection: u64, identity: u64) {
    let key = SigningKey::from_bytes(&[identity as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{identity}"),
        reconnect_token: ReconnectToken(identity),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    send(session, connection, ClientCommand::join(request));
    send(
        session,
        connection,
        ClientCommand::SelectSeat {
            seat: SeatId(identity as u8 - 1),
        },
    );
    send(session, connection, ClientCommand::SetReady { ready: true });
}

fn act(
    session: &mut MahjongLinkSession,
    connection: u64,
    action: MahjongLinkAction,
) -> Vec<Delivery> {
    send(
        session,
        connection,
        ClientCommand::Game(MahjongLinkCommand::Act { action }.into()),
    )
}

fn lobby() -> MahjongLinkSession {
    MahjongLinkSession::new(
        RoomId(99),
        52300,
        MahjongLinkRuleSet::default(),
        build_deck(),
    )
    .unwrap()
}

#[test]
fn host_rejects_other_turns_and_out_of_bounds_without_changing_board() {
    let mut session = lobby();
    join(&mut session, 1);
    join(&mut session, 2);
    send(&mut session, 1, ClientCommand::StartGame);
    assert!(session.game.is_some());
    let before = session.game.as_ref().unwrap().board().clone();
    let replies = act(&mut session, 2, MahjongLinkAction::Pass);
    assert!(replies.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::MahjongLink(MahjongLinkViolation(
                MahjongLinkError::NotPlayersTurn
            )))
        }
    )));
    let replies = act(
        &mut session,
        1,
        MahjongLinkAction::Push {
            row: 0,
            from: PushSide::Left,
            distance: 1,
        },
    );
    assert!(replies.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::MahjongLink(MahjongLinkViolation(
                MahjongLinkError::PushOutOfBounds
            )))
        }
    )));
    assert_eq!(session.game.as_ref().unwrap().board(), &before);
    assert_eq!(session.game.as_ref().unwrap().turn(), 0);
    let replies = act(
        &mut session,
        1,
        MahjongLinkAction::Match {
            first: LinkCell { row: 0, column: 0 },
            second: LinkCell { row: 0, column: 1 },
        },
    );
    let snapshots: Vec<_> = replies
        .iter()
        .filter_map(|delivery| {
            if let ServerEvent::GameSnapshot(GameSnapshot::MahjongLink(game)) =
                &delivery.message.event
            {
                Some(game)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].board, snapshots[1].board);
    assert_eq!(snapshots[0].players[0].score, 1);
    assert_eq!(snapshots[0].current_player, Some(session.order[1]));
    assert_eq!(snapshots[0].board.remaining(), 134);
    let message = snapshots_message(&session);
    let encoded = leocard_protocol::encode_frame(&message).unwrap();
    let decoded: leocard_protocol::ServerMessage =
        leocard_protocol::decode_frame(&encoded).unwrap();
    assert_eq!(decoded, message);

    join(&mut session, 3);
    let host = session.order[0];
    let replies = send(
        &mut session,
        3,
        ClientCommand::SpectatePlayer { player: host },
    );
    let spectator = replies
        .iter()
        .find_map(|delivery| {
            if delivery.recipient == ConnectionId(3)
                && let ServerEvent::GameSnapshot(GameSnapshot::MahjongLink(game)) =
                    &delivery.message.event
            {
                Some(game)
            } else {
                None
            }
        })
        .expect("spectator receives a snapshot");
    assert!(
        !spectator
            .players
            .iter()
            .any(|player| player.player.id == spectator.you)
    );
    assert_ne!(spectator.current_player, Some(spectator.you));
    let before = spectator.board.clone();
    let replies = act(&mut session, 3, MahjongLinkAction::Pass);
    assert!(replies.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::SpectatorCannotAct)
        }
    )));
    assert_eq!(session.game.as_ref().unwrap().board(), &before);
}

fn snapshots_message(session: &MahjongLinkSession) -> leocard_protocol::ServerMessage {
    session
        .room
        .delivery(
            ConnectionId(1),
            None,
            ServerEvent::GameSnapshot(session.snapshot_for(session.order[0]).into()),
        )
        .message
}

#[test]
fn pushing_phase_survives_reconnect_and_rejects_another_push() {
    let mut session = lobby();
    join(&mut session, 1);
    join(&mut session, 2);
    send(&mut session, 1, ClientCommand::StartGame);
    act(
        &mut session,
        1,
        MahjongLinkAction::Match {
            first: LinkCell { row: 0, column: 0 },
            second: LinkCell { row: 0, column: 1 },
        },
    );
    let push = MahjongLinkAction::Push {
        row: 0,
        from: PushSide::Right,
        distance: 1,
    };
    act(&mut session, 2, push);
    let player = session.order[1];
    let before = session.snapshot_for(player);
    assert!(before.push_used);
    assert_eq!(before.current_player, Some(player));
    assert_eq!(before.turn, 1);
    session.disconnect(ConnectionId(2));
    join_as(&mut session, 20, 2);
    let after = session.snapshot_for(player);
    assert!(after.push_used);
    assert_eq!(after.board, before.board);
    let message = snapshots_message(&session);
    let encoded = leocard_protocol::encode_frame(&message).unwrap();
    assert_eq!(
        leocard_protocol::decode_frame::<leocard_protocol::ServerMessage>(&encoded).unwrap(),
        message
    );
    let replies = act(&mut session, 20, push);
    assert!(replies.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::MahjongLink(MahjongLinkViolation(
                MahjongLinkError::PushAlreadyUsed
            )))
        }
    )));
    act(
        &mut session,
        20,
        MahjongLinkAction::Match {
            first: LinkCell { row: 0, column: 1 },
            second: LinkCell { row: 0, column: 2 },
        },
    );
    let next = session.snapshot_for(player);
    assert!(!next.push_used);
    assert_eq!(next.turn, 2);
    assert_eq!(next.players[1].score, 1);
    assert_eq!(next.current_player, Some(session.order[0]));
}

#[test]
fn actual_seated_count_controls_turns_and_rematches_up_to_eight() {
    for count in MahjongLinkRuleSet::MIN_PLAYERS..=MahjongLinkRuleSet::MAX_PLAYERS {
        let mut session = lobby();
        assert_eq!(session.room.seat_count, MahjongLinkRuleSet::MAX_PLAYERS);
        for connection in 1..=u64::from(count) {
            join(&mut session, connection);
        }
        let started = send(&mut session, 1, ClientCommand::StartGame);
        assert!(
            started
                .iter()
                .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
        let match_id = session.match_id;
        assert_eq!(session.order.len(), usize::from(count));
        assert_eq!(
            session.game.as_ref().unwrap().scores(),
            vec![0; usize::from(count)]
        );
        for connection in 1..=u64::from(count) {
            act(&mut session, connection, MahjongLinkAction::Pass);
        }
        assert!(session.game.as_ref().unwrap().complete());
        let next_count = if count > 1 {
            send(&mut session, u64::from(count), ClientCommand::LeaveRoom);
            count - 1
        } else {
            count
        };
        for connection in 1..=u64::from(next_count) {
            send(&mut session, connection, ClientCommand::PlayAgain);
        }
        assert_ne!(session.match_id, match_id);
        assert_eq!(session.order.len(), usize::from(next_count));
        assert_eq!(session.game.as_ref().unwrap().player_count(), next_count);
        assert_eq!(session.game.as_ref().unwrap().board().remaining(), 136);
    }
    let mut full = lobby();
    for connection in 1..=9 {
        join(&mut full, connection);
    }
    assert_eq!(
        full.room
            .players
            .iter()
            .filter(|player| !player.left)
            .count(),
        8
    );
}

#[test]
fn disconnected_turn_is_automated_and_rematch_starts_fresh() {
    let mut session = lobby();
    join(&mut session, 1);
    join(&mut session, 2);
    send(&mut session, 1, ClientCommand::StartGame);
    let first_match = session.match_id;
    act(&mut session, 1, MahjongLinkAction::Pass);
    session.disconnect(ConnectionId(2));
    session.advance_time(Duration::from_secs(2));
    assert_eq!(session.game.as_ref().unwrap().turn(), 2);
    assert_eq!(session.game.as_ref().unwrap().current_player(), Some(0));
    // Reconnect retains the authoritative board and score.
    join_as(&mut session, 20, 2);
    assert!(!session.is_current_connection(ConnectionId(2)));
    assert!(session.is_current_connection(ConnectionId(20)));
    let score = session.game.as_ref().unwrap().scores()[1];
    assert!(score > 0);
    act(&mut session, 1, MahjongLinkAction::Pass);
    act(&mut session, 20, MahjongLinkAction::Pass);
    assert!(session.game.as_ref().unwrap().complete());
    send(&mut session, 1, ClientCommand::PlayAgain);
    send(&mut session, 20, ClientCommand::PlayAgain);
    assert_ne!(session.match_id, first_match);
    let game = session.game.as_ref().unwrap();
    assert_eq!(game.board().remaining(), 136);
    assert_eq!(game.scores(), &[0, 0]);
    assert_eq!(game.turn(), 0);
}
