use super::DoudizhuSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_doudizhu::{Action, GameError, Mode, Phase, RuleSet, bot_action, build_deck};
use leocard_protocol::{
    ClientCommand, ClientMessage, DoudizhuCommand, GameSnapshot, GameViolation, JoinRequest,
    PlayerGameProfiles, PlayerId, ProfileId, ReconnectToken, RejectReason, RequestId, RoomId,
    SeatId, ServerEvent,
};
use std::time::Duration;

fn send(session: &mut DoudizhuSession, connection: u64, command: ClientCommand) -> Vec<Delivery> {
    let connection = ConnectionId(connection);
    let request = session
        .room
        .last_requests
        .get(&connection)
        .map_or(1, |r| r.0 + 1);
    session.handle(
        connection,
        ClientMessage::new(RoomId(99), RequestId(request), command),
    )
}

fn join(connection: u64) -> ClientCommand {
    let key = SigningKey::from_bytes(&[connection as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{connection}"),
        reconnect_token: ReconnectToken(connection),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    ClientCommand::join(request)
}

fn assert_accepted(deliveries: &[Delivery]) {
    for delivery in deliveries {
        assert!(
            !matches!(delivery.message.event, ServerEvent::Rejected { .. }),
            "{:?}",
            delivery.message.event
        );
    }
}

fn started(mode: Mode, robots: bool) -> DoudizhuSession {
    let mut session =
        DoudizhuSession::new(RoomId(99), 52300, RuleSet { mode }, build_deck(mode)).unwrap();
    for connection in 1..=if robots { 1 } else { u64::from(mode.players()) } {
        assert_accepted(&send(&mut session, connection, join(connection)));
        assert_accepted(&send(
            &mut session,
            connection,
            ClientCommand::SelectSeat {
                seat: SeatId(connection as u8 - 1),
            },
        ));
        assert_accepted(&send(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        ));
    }
    if robots {
        for seat in 1..mode.players() {
            assert_accepted(&send(
                &mut session,
                1,
                ClientCommand::ConfigureBotSeat {
                    seat: SeatId(seat),
                    occupied: true,
                },
            ));
        }
    }
    assert_accepted(&send(&mut session, 1, ClientCommand::StartGame));
    assert!(session.game.is_some());
    session
}

fn act(session: &mut DoudizhuSession, player: u8, action: Action) -> Vec<Delivery> {
    let connection = session.room.players[usize::from(player)].connection.0;
    send(
        session,
        connection,
        ClientCommand::Game(DoudizhuCommand::Act(action).into()),
    )
}

#[test]
fn manual_turns_never_expire_and_snapshots_keep_hands_private() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, false);
        let current = session.game.as_ref().unwrap().current;
        let revision = session.revision();
        assert!(session.advance_time(Duration::from_secs(86400)).is_empty());
        assert_eq!(session.revision(), revision);
        assert_eq!(session.game.as_ref().unwrap().current, current);
        assert_accepted(&act(&mut session, current, Action::Bid(1)));
        for player in 0..mode.players() {
            let snapshot = session.snapshot_for(PlayerId(player));
            assert_eq!(
                snapshot.your_hand,
                session.game.as_ref().unwrap().hand(player).unwrap()
            );
            assert!(snapshot.kitty.is_empty());
            assert!(snapshot.revealed_hands.is_empty());
        }
        let current = session.game.as_ref().unwrap().current;
        assert_accepted(&act(&mut session, current, Action::Bid(3)));
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Playing);
        assert_eq!(
            session.snapshot_for(PlayerId(current)).kitty.len(),
            mode.kitty_len()
        );
        assert!(session.advance_time(Duration::from_secs(86400)).is_empty());
        assert!(session.game.as_ref().unwrap().played.is_empty());
        let other = (current + 1) % mode.players();
        let rejected = act(&mut session, other, Action::Pass);
        assert!(rejected.iter().any(|d| matches!(
            d.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Doudizhu(GameError::NotYourTurn))
            }
        )));
        assert!(session.game.as_ref().unwrap().played.is_empty());
        assert_accepted(&send(&mut session, 9, join(9)));
        let spectator = session.room.player_id(ConnectionId(9)).unwrap();
        assert!(session.snapshot_for(spectator).your_hand.is_empty());
    }
}

#[test]
fn robots_play_both_modes_rematch_and_return_to_lobby() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, true);
        let match_id = session.match_id;
        assert_accepted(&send(
            &mut session,
            1,
            ClientCommand::Game(DoudizhuCommand::SetAutoPlay { enabled: true }.into()),
        ));
        for _ in 0..3000 {
            if session.game.as_ref().unwrap().phase == Phase::Finished {
                break;
            }
            session.advance_time(Duration::from_secs(1));
        }
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Finished);
        let snapshot = session.snapshot_for(PlayerId(0));
        assert!(snapshot.players.iter().all(|player| player.ready));
        assert_eq!(snapshot.revealed_hands.len(), usize::from(mode.players()));
        assert_eq!(
            snapshot.result.as_ref().unwrap().scores.iter().sum::<i32>(),
            0
        );
        assert_accepted(&send(&mut session, 1, ClientCommand::PlayAgain));
        assert_ne!(session.match_id, match_id);
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Bidding);
        assert!(!session.room.players[0].auto_play);
        for _ in 0..3000 {
            let game = session.game.as_ref().unwrap();
            if game.phase == Phase::Finished {
                break;
            }
            let current = game.current;
            let action = bot_action(game).unwrap();
            // The same legal rules interface is used for the human and robot moves.
            session.apply_action(current, action, None);
        }
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Finished);
        assert_accepted(&send(&mut session, 1, ClientCommand::ReturnToLobby));
        assert!(session.game.is_none());
        assert_eq!(session.room.players.iter().filter(|p| !p.left).count(), 1);
    }
}

#[test]
fn disconnect_uses_automation_and_reconnect_restores_own_hand() {
    let mut session = started(Mode::Three, false);
    // Make player two the landlord so disconnecting does not close the host room.
    while session.game.as_ref().unwrap().current != 1 {
        let current = session.game.as_ref().unwrap().current;
        assert_accepted(&act(&mut session, current, Action::Bid(0)));
    }
    assert_accepted(&act(&mut session, 1, Action::Bid(3)));
    session.disconnect(ConnectionId(2));
    let before = session.game.as_ref().unwrap().hand(1).unwrap().len();
    session.advance_time(Duration::from_secs(1));
    assert!(session.game.as_ref().unwrap().hand(1).unwrap().len() < before);
    let expected = session.game.as_ref().unwrap().hand(1).unwrap().to_vec();
    let deliveries = send(&mut session, 22, join(2));
    assert_accepted(&deliveries);
    let snapshot = deliveries
        .iter()
        .find_map(|d| match &d.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Doudizhu(snapshot))
                if d.recipient == ConnectionId(22) =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(snapshot.your_hand, expected);
    assert!(snapshot.revealed_hands.is_empty());
    assert!(!session.is_current_connection(ConnectionId(2)));
}
