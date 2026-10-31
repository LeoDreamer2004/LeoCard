use super::BlackjackSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_blackjack::{
    BlackjackAction, BlackjackPhase, BlackjackRank, BlackjackRuleSet, build_shoe,
};
use leocard_protocol::{
    BlackjackCommand, BlackjackViolation, ChatContent, ClientCommand, ClientMessage, GameSnapshot,
    GameViolation, JoinRequest, PlayerGameProfiles, PlayerInteractionKind, ProfileId,
    ReconnectToken, RejectReason, RequestId, RoomId, SeatId, ServerEvent,
};
use std::time::Duration;

fn send(session: &mut BlackjackSession, connection: u64, command: ClientCommand) -> Vec<Delivery> {
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

fn act(session: &mut BlackjackSession, connection: u64, action: BlackjackAction) -> Vec<Delivery> {
    send(
        session,
        connection,
        ClientCommand::Game(BlackjackCommand::Act { action }.into()),
    )
}

fn started(robots: bool) -> BlackjackSession {
    let mut remaining = build_shoe();
    let mut shoe = Vec::new();
    for rank in [
        BlackjackRank::Ten,
        BlackjackRank::Nine,
        BlackjackRank::Six,
        BlackjackRank::Eight,
        BlackjackRank::Ten,
        BlackjackRank::Ten,
        BlackjackRank::Five,
    ] {
        let index = remaining.iter().position(|card| card.rank == rank).unwrap();
        shoe.push(remaining.remove(index));
    }
    shoe.extend(remaining);
    let mut session = BlackjackSession::new(
        RoomId(99),
        52300,
        BlackjackRuleSet {
            rounds: 1,
            ..Default::default()
        },
        shoe,
    )
    .unwrap();
    for connection in 1..=if robots { 1 } else { 2 } {
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
        send(&mut session, connection, ClientCommand::join(request));
        // The first participant's random seat may otherwise occupy the second seat.
        send(
            &mut session,
            connection,
            ClientCommand::SelectSeat {
                seat: SeatId(connection as u8 - 1),
            },
        );
        send(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        );
    }
    if robots {
        send(
            &mut session,
            1,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(1),
                occupied: true,
            },
        );
    }
    let started = send(&mut session, 1, ClientCommand::StartGame);
    let announcements = started
        .iter()
        .filter(|delivery| matches!(delivery.message.event, ServerEvent::GameStarted { .. }))
        .collect::<Vec<_>>();
    assert!(!announcements.is_empty());
    assert!(announcements.iter().all(|delivery| {
        delivery.message.game_context.is_some_and(|context| {
            context.records_progress == (cfg!(feature = "developer") || !robots)
        })
    }));
    session
}

#[test]
fn host_rejects_out_of_turn_actions_without_resetting_time_and_keeps_hole_card_private() {
    let mut session = started(false);
    session.advance_time(Duration::from_secs(5));
    let rejected = act(&mut session, 2, BlackjackAction::Bet { amount: 2 });
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::Blackjack(
                BlackjackViolation::NotPlayersTurn
            ))
        }
    )));
    assert_eq!(session.seconds_remaining(), Some(25));
    act(&mut session, 1, BlackjackAction::Bet { amount: 2 });
    let dealt = act(&mut session, 2, BlackjackAction::Bet { amount: 2 });
    for delivery in dealt {
        if let ServerEvent::GameSnapshot(GameSnapshot::Blackjack(snapshot)) = delivery.message.event
        {
            assert!(snapshot.dealer_hidden);
            assert_eq!(snapshot.dealer.len(), 1);
            assert_eq!(snapshot.dealer[0].rank, BlackjackRank::Six);
            assert_eq!(
                snapshot.available.stand,
                snapshot.you == snapshot.current_player.unwrap()
            );
        }
    }
    act(&mut session, 1, BlackjackAction::Stand);
    act(&mut session, 2, BlackjackAction::Stand);
    assert_eq!(
        session.game.as_ref().unwrap().phase(),
        BlackjackPhase::Complete
    );
    let finished = session.snapshot_for(session.order[0]);
    assert!(!finished.dealer_hidden);
    assert_eq!(finished.dealer.len(), 3);
    assert!(finished.match_complete);
    assert!(session.room.players.iter().all(|player| {
        player
            .game_profiles
            .blackjack
            .as_ref()
            .unwrap()
            .statistics
            .hands
            == 1
    }));
    session.settle_match();
    send(&mut session, 1, ClientCommand::RequestSnapshot);
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.completed_games == 1)
    );
}

#[test]
fn robot_round_uses_shared_recording_policy_and_waits_for_human_rematch_ready() {
    let mut session = started(true);
    let target = session.room.players[1].id;
    for command in [
        ClientCommand::Chat {
            content: ChatContent::QuickVoice(0),
        },
        ClientCommand::Interact {
            target,
            kind: PlayerInteractionKind::Flower,
        },
    ] {
        let deliveries = send(&mut session, 1, command);
        assert!(!deliveries.is_empty());
        assert!(deliveries.iter().all(|delivery| {
            delivery
                .message
                .game_context
                .is_some_and(|context| context.records_progress == cfg!(feature = "developer"))
        }));
    }
    act(&mut session, 1, BlackjackAction::Bet { amount: 2 });
    session.advance_time(Duration::from_secs(2));
    act(&mut session, 1, BlackjackAction::Stand);
    session.advance_time(Duration::from_secs(2));
    assert!(session.game.as_ref().unwrap().match_complete());
    let expected_games = u32::from(cfg!(feature = "developer"));
    assert_eq!(session.room.players[0].completed_games, expected_games);
    assert_eq!(
        session.room.players[0].game_profiles.blackjack.is_some(),
        cfg!(feature = "developer")
    );
    let previous = session.match_id;
    assert!(session.advance_time(Duration::from_secs(20)).is_empty());
    send(&mut session, 1, ClientCommand::PlayAgain);
    assert_ne!(session.match_id, previous);
    assert_eq!(
        session.game.as_ref().unwrap().phase(),
        BlackjackPhase::Betting
    );
    assert_eq!(session.room.players[0].completed_games, expected_games);
}
