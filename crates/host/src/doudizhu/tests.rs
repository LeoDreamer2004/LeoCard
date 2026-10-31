use leocard_game_common::PlayerId;
mod achievements;
mod modes;

use super::DoudizhuSession;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_doudizhu::{
    DoudizhuAction, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, bot_action, build_deck,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, DoudizhuCommand, DoudizhuViolation, GameViolation, JoinRequest,
    PlayerGameProfiles, ProfileId, ReconnectToken, RejectReason, RequestId, RoomId, SeatId,
    ServerEvent,
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

fn started(mode: DoudizhuMode, robots: bool) -> DoudizhuSession {
    let mut session = DoudizhuSession::new(
        RoomId(99),
        52300,
        DoudizhuRuleSet {
            mode,
            ..Default::default()
        },
        build_deck(mode),
    )
    .unwrap();
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

fn act(session: &mut DoudizhuSession, player: PlayerId, action: DoudizhuAction) -> Vec<Delivery> {
    let connection = session.room.players[usize::from(player)].connection.0;
    send(
        session,
        connection,
        ClientCommand::Game(DoudizhuCommand::Act(action).into()),
    )
}

#[test]
fn manual_turns_never_expire_and_snapshots_keep_hands_private() {
    for mode in [DoudizhuMode::Three, DoudizhuMode::Four] {
        let mut session = started(mode, false);
        let current = session.game.as_ref().unwrap().current;
        let revision = session.revision();
        assert!(session.advance_time(Duration::from_secs(86400)).is_empty());
        assert_eq!(session.revision(), revision);
        assert_eq!(session.game.as_ref().unwrap().current, current);
        assert_accepted(&act(&mut session, current, DoudizhuAction::Bid(1)));
        for player in (0..mode.players()).map(PlayerId) {
            let snapshot = session.snapshot_for(player);
            assert_eq!(
                snapshot.your_hand,
                session.game.as_ref().unwrap().hand(player).unwrap()
            );
            assert!(snapshot.kitty.is_empty());
            assert!(snapshot.revealed_hands.is_empty());
        }
        let current = session.game.as_ref().unwrap().current;
        assert_accepted(&act(&mut session, current, DoudizhuAction::Bid(3)));
        assert_eq!(session.game.as_ref().unwrap().phase, DoudizhuPhase::Playing);
        assert_eq!(session.snapshot_for(current).kitty.len(), mode.kitty_len());
        assert!(session.advance_time(Duration::from_secs(86400)).is_empty());
        assert!(session.game.as_ref().unwrap().played.is_empty());
        let other = PlayerId((current.0 + 1) % mode.players());
        let rejected = act(&mut session, other, DoudizhuAction::Pass);
        assert!(rejected.iter().any(|d| matches!(
            d.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Doudizhu(DoudizhuViolation::NotYourTurn))
            }
        )));
        assert!(session.game.as_ref().unwrap().played.is_empty());
        assert_accepted(&send(&mut session, 9, join(9)));
        let spectator = session.room.player_id(ConnectionId(9)).unwrap();
        assert!(session.snapshot_for(spectator).your_hand.is_empty());
    }
}

fn finish_hand(session: &mut DoudizhuSession) {
    for _ in 0..3000 {
        let game = session.game.as_ref().unwrap();
        if game.phase == DoudizhuPhase::Finished {
            return;
        }
        let current = game.current;
        let action = bot_action(game).unwrap();
        session.apply_action(current, action, None);
    }
    panic!("bots did not finish a hand");
}

fn ready_players(session: &mut DoudizhuSession) {
    let connections: Vec<_> = session
        .room
        .players
        .iter()
        .filter(|p| !p.is_bot && !p.left && p.connected)
        .map(|p| p.connection.0)
        .collect();
    for connection in connections {
        assert_accepted(&send(session, connection, ClientCommand::PlayAgain));
    }
}

#[test]
fn all_passes_redeal_without_advancing_and_profiles_settle_only_at_match_end() {
    for mode in [DoudizhuMode::Three, DoudizhuMode::Four] {
        let mut session = started(mode, false);
        let first = session.game.as_ref().unwrap().current;
        for _ in 0..mode.players() {
            let player = session.game.as_ref().unwrap().current;
            assert_accepted(&act(&mut session, player, DoudizhuAction::Bid(0)));
        }
        assert_eq!(session.game.as_ref().unwrap().current, first);
        assert_eq!(session.snapshot_for(PlayerId(0)).progress.hand_number, 1);
        let mut totals = vec![0; usize::from(mode.players())];
        for hand in 0..mode.players() {
            finish_hand(&mut session);
            let snapshot = session.snapshot_for(PlayerId(0));
            for (total, score) in totals.iter_mut().zip(&snapshot.result.unwrap().scores) {
                *total += score;
            }
            assert_eq!(snapshot.scores, totals);
            if hand + 1 < mode.players() {
                assert!(snapshot.reference_changes.is_empty());
                assert!(
                    session
                        .room
                        .players
                        .iter()
                        .all(|p| p.completed_games == 0 && p.reference_points == 0)
                );
                ready_players(&mut session);
            }
        }
        let snapshot = session.snapshot_for(PlayerId(0));
        let changes = snapshot.reference_changes;
        assert_eq!(changes.len(), usize::from(mode.players()));
        assert!(session.room.players.iter().all(|p| p.completed_games == 1));
        for change in &changes {
            assert_eq!(
                session.room.players[usize::from(change.player.0)].reference_points,
                i32::from(change.delta)
            );
        }
        let archived = session
            .room
            .players
            .iter()
            .map(|player| player.game_profiles.doudizhu.clone().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            archived
                .iter()
                .map(|stats| stats.landlord_hands)
                .sum::<u32>(),
            u32::from(mode.players())
        );
        for (index, stats) in archived.iter().enumerate() {
            assert_eq!(stats.completed_games, 1);
            assert_eq!(stats.hands_played, u32::from(mode.players()));
            assert_eq!(stats.total_match_score, i64::from(totals[index]));
            assert!(stats.bid_actions >= 1); // The initial all-pass deal counted a real zero bid.
            assert_eq!(stats.total_reference_delta, i64::from(changes[index].delta));
        }
        session.advance_time(Duration::from_secs(60));
        assert_eq!(
            session
                .room
                .players
                .iter()
                .map(|player| player.game_profiles.doudizhu.clone().unwrap())
                .collect::<Vec<_>>(),
            archived
        );
        assert_eq!(session.snapshot_for(PlayerId(0)).reference_changes, changes);
        assert!(session.room.players.iter().all(|p| p.completed_games == 1));
    }
}
