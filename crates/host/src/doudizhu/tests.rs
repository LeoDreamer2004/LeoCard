mod achievements;
mod modes;

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
    let mut session = DoudizhuSession::new(
        RoomId(99),
        52300,
        RuleSet {
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

fn finish_hand(session: &mut DoudizhuSession) {
    for _ in 0..3000 {
        let game = session.game.as_ref().unwrap();
        if game.phase == Phase::Finished {
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
fn robots_play_all_hands_rotate_first_bidder_rematch_and_return_to_lobby() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, true);
        let match_id = session.match_id;
        let first = session.game.as_ref().unwrap().current;
        for hand in 0..mode.players() {
            assert_eq!(
                session.game.as_ref().unwrap().current,
                (first + hand) % mode.players()
            );
            finish_hand(&mut session);
            let snapshot = session.snapshot_for(PlayerId(0));
            assert_eq!(snapshot.progress.hand_number, hand + 1);
            assert_eq!(snapshot.match_complete, hand == mode.players() - 1);
            assert_eq!(snapshot.revealed_hands.len(), usize::from(mode.players()));
            assert_eq!(snapshot.scores.iter().sum::<i32>(), 0);
            let records = snapshot.match_complete && cfg!(feature = "developer");
            assert_eq!(
                snapshot.reference_changes.len(),
                if records {
                    usize::from(mode.players())
                } else {
                    0
                }
            );
            assert_eq!(session.room.players[0].completed_games, u32::from(records));
            assert_eq!(
                session.room.players[0].game_profiles.doudizhu.is_some(),
                records
            );
            if !snapshot.match_complete {
                ready_players(&mut session);
                assert_eq!(session.match_id, match_id);
            }
        }
        ready_players(&mut session);
        assert_ne!(session.match_id, match_id);
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Bidding);
        assert!(
            session
                .statistics
                .players
                .iter()
                .all(|stats| *stats == Default::default())
        );
        assert!(
            session
                .match_state
                .as_ref()
                .unwrap()
                .scores()
                .iter()
                .all(|score| *score == 0)
        );
        for hand in 0..mode.players() {
            finish_hand(&mut session);
            if hand + 1 < mode.players() {
                ready_players(&mut session);
            }
        }
        assert_accepted(&send(&mut session, 1, ClientCommand::ReturnToLobby));
        assert!(session.game.is_none());
        assert_eq!(session.room.players.iter().filter(|p| !p.left).count(), 1);
    }
}

#[test]
fn all_passes_redeal_without_advancing_and_profiles_settle_only_at_match_end() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, false);
        let first = session.game.as_ref().unwrap().current;
        for _ in 0..mode.players() {
            let player = session.game.as_ref().unwrap().current;
            assert_accepted(&act(&mut session, player, Action::Bid(0)));
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

#[test]
fn bidding_statistics_count_passes_but_not_players_skipped_by_a_three_bid() {
    for mode in [Mode::Three, Mode::Four] {
        let mut session = started(mode, false);
        for _ in 0..mode.players() {
            let current = session.game.as_ref().unwrap().current;
            assert_accepted(&act(&mut session, current, Action::Bid(0)));
        }
        // All passes redeal the same hand; these accepted zero bids remain recorded.
        let current = session.game.as_ref().unwrap().current;
        assert_accepted(&act(&mut session, current, Action::Bid(3)));
        assert_eq!(session.game.as_ref().unwrap().phase, Phase::Playing);
        for (player, stats) in session.statistics.players.iter().enumerate() {
            let called_three = player == usize::from(current);
            assert_eq!(stats.bid_actions, if called_three { 2 } else { 1 });
            assert_eq!(stats.total_bid_score, if called_three { 3 } else { 0 });
            assert_eq!(stats.hands_played, 0);
        }
        let before = session.statistics.players.clone();
        act(&mut session, current, Action::Bid(1));
        assert_eq!(session.statistics.players, before);
        assert!(
            session
                .room
                .players
                .iter()
                .all(|player| player.game_profiles.doudizhu.is_none())
        );
    }
}

#[test]
fn pattern_statistics_count_body_lengths_and_combine_both_rocket_types_with_bombs() {
    use super::statistics::DoudizhuStatistics;
    use leocard_doudizhu::{Card, classify};
    use leocard_protocol::DoudizhuProfileStats;

    let mut stats = DoudizhuProfileStats::default();
    let patterns = [
        (Mode::Three, vec![0, 4, 8, 12, 16, 20]),
        (Mode::Three, vec![0, 1, 4, 5, 8, 9]),
        (Mode::Three, vec![0, 1, 2, 4, 5, 6]),
        (Mode::Three, vec![0, 1, 2, 4, 5, 6, 8, 12]),
        (Mode::Three, vec![0, 1, 2, 4, 5, 6, 8, 9, 12, 13]),
        (Mode::Three, vec![0, 1, 2, 3]),
        (Mode::Three, vec![52, 53]),
        (Mode::Four, vec![0, 1, 2, 3]),
        (Mode::Four, vec![52, 53, 106, 107]),
    ];
    for (mode, cards) in patterns {
        let play = classify(
            &cards.into_iter().map(Card).collect::<Vec<_>>(),
            RuleSet {
                mode,
                ..Default::default()
            },
        )
        .unwrap();
        DoudizhuStatistics::observe_play(&mut stats, &play);
    }
    assert_eq!((stats.straight_plays, stats.longest_straight), (1, 6));
    assert_eq!(
        (
            stats.consecutive_pair_plays,
            stats.longest_consecutive_pairs
        ),
        (1, 3)
    );
    assert_eq!((stats.airplane_plays, stats.longest_airplane), (3, 2));
    assert_eq!(stats.bomb_plays, 4);
}
