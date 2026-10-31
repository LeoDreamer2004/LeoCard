use super::super::McGuessSession;
use super::support::{join, send, snapshot};
use crate::ConnectionId;
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessOptions, McGuessPool,
    McGuessRuleSet, McGuessTurnBasedRules,
};
use leocard_protocol::{
    ClientCommand, GameEvent, GameViolation, McGuessCommand, McGuessEvent, RejectReason, RoomId,
    SeatId, ServerEvent,
};
use std::time::Duration;

#[test]
fn all_players_receive_feedback_then_can_rematch_and_return_to_lobby() {
    let rules = McGuessRuleSet {
        turn_based: McGuessTurnBasedRules {
            options: McGuessOptions {
                pool: McGuessPool::Blocks,
                candidate_hints: false,
                allow_unknown_guesses: true,
            },
            ..McGuessRuleSet::default().turn_based
        },
        ..McGuessRuleSet::default()
    };
    let mut session = McGuessSession::new(RoomId(99), 52300, rules);
    for connection in [1, 2] {
        join(&mut session, connection);
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
    let invalid_rules = send(
        &mut session,
        1,
        ClientCommand::Game(
            McGuessCommand::UpdateRules {
                rules: McGuessRuleSet {
                    turn_based: McGuessTurnBasedRules {
                        options: McGuessOptions {
                            candidate_hints: true,
                            ..rules.turn_based.options
                        },
                        ..rules.turn_based
                    },
                    ..rules
                },
            }
            .into(),
        ),
    );
    assert!(matches!(
        invalid_rules[0].message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            ..
        }
    ));
    assert_eq!(session.rules, rules);
    let deliveries = send(&mut session, 1, ClientCommand::StartGame);
    assert!(snapshot(&deliveries, 1).round.answer.is_none());
    assert!(snapshot(&deliveries, 2).round.answer.is_none());
    // Deterministic target exercises the same authoritative command path as random rounds.
    let target = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:oak_log")
        .unwrap()
        .clone();
    session.game = Some(McGuessGameState::new(
        target,
        vec![PlayerId(0), PlayerId(1)],
        rules,
    ));
    let wrong_turn = send(
        &mut session,
        2,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    assert!(matches!(
        wrong_turn[0].message.event,
        ServerEvent::Rejected { .. }
    ));
    let deliveries = send(
        &mut session,
        1,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: "不在词库中".to_owned(),
            }
            .into(),
        ),
    );
    assert_eq!(
        snapshot(&deliveries, 1).round,
        snapshot(&deliveries, 2).round
    );
    let facts: Vec<_> = deliveries
        .iter()
        .filter(|delivery| {
            matches!(
                delivery.message.event,
                ServerEvent::GameEvent(GameEvent::McGuess(_))
            )
        })
        .collect();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].recipient, ConnectionId(1));
    assert!(facts[0].message.game_context.is_some());
    assert_eq!(snapshot(&deliveries, 2).round.guesses.len(), 1);
    assert_eq!(
        snapshot(&deliveries, 2).round.guesses[0].result.name(),
        "不在词库中"
    );
    assert_eq!(
        snapshot(&deliveries, 2).round.current_player,
        Some(PlayerId(1))
    );
    assert!(snapshot(&deliveries, 2).round.answer.is_none());
    let deliveries = send(
        &mut session,
        2,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: "minecraft:oak_log".to_owned(),
            }
            .into(),
        ),
    );
    assert_eq!(snapshot(&deliveries, 1).round.winner, Some(PlayerId(1)));
    assert_eq!(
        deliveries
            .iter()
            .filter(|delivery| {
                matches!(
                    delivery.message.event,
                    ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::HandAnalyzed { .. }))
                )
            })
            .count(),
        2
    );
    let first_match = snapshot(&deliveries, 1).match_id;
    send(&mut session, 1, ClientCommand::PlayAgain);
    let next = send(&mut session, 2, ClientCommand::PlayAgain);
    assert_eq!(snapshot(&next, 1).match_id, first_match);
    assert_eq!(snapshot(&next, 1).progress.game_index, 1);
    assert_eq!(snapshot(&next, 1).round.current_player, Some(PlayerId(1)));
    assert!(snapshot(&next, 1).round.answer.is_none());
    send(
        &mut session,
        2,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    send(
        &mut session,
        1,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    let lobby = send(&mut session, 1, ClientCommand::ReturnToLobby);
    assert!(
        lobby
            .iter()
            .all(|delivery| matches!(delivery.message.event, ServerEvent::LobbySnapshot(_)))
    );
}

#[test]
fn bots_can_start_and_disconnected_turns_do_not_block_the_game() {
    let mut session = McGuessSession::new(
        RoomId(99),
        52300,
        McGuessRuleSet {
            turn_based: McGuessTurnBasedRules {
                options: McGuessOptions {
                    candidate_hints: false,
                    allow_unknown_guesses: true,
                    ..McGuessRuleSet::default().turn_based.options
                },
                ..McGuessRuleSet::default().turn_based
            },
            ..McGuessRuleSet::default()
        },
    );
    join(&mut session, 1);
    send(
        &mut session,
        1,
        ClientCommand::SelectSeat { seat: SeatId(0) },
    );
    send(
        &mut session,
        1,
        ClientCommand::ConfigureBotSeat {
            seat: SeatId(1),
            occupied: true,
        },
    );
    let start = send(&mut session, 1, ClientCommand::StartGame);
    assert_eq!(snapshot(&start, 1).round.current_player, Some(PlayerId(0)));
    send(
        &mut session,
        1,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: "不存在的词".to_owned(),
            }
            .into(),
        ),
    );
    let countdown = session.advance_time(Duration::from_secs(1));
    assert_eq!(snapshot(&countdown, 1).guess_seconds_remaining, Some(59));
    assert_eq!(snapshot(&countdown, 1).round.guesses.len(), 1);
    let feedback = session.advance_time(Duration::from_secs(1));
    assert_eq!(snapshot(&feedback, 1).round.guesses.len(), 2);
}

#[test]
fn scoring_settles_profiles_once_at_match_end_and_resets_for_next_match() {
    let rules = McGuessRuleSet {
        turn_based: McGuessTurnBasedRules {
            options: McGuessOptions {
                allow_unknown_guesses: true,
                candidate_hints: false,
                ..McGuessRuleSet::default().turn_based.options
            },
            ..McGuessRuleSet::default().turn_based
        },
        ..McGuessRuleSet::default()
    };
    let mut session = McGuessSession::new(RoomId(99), 52300, rules);
    for connection in [1, 2] {
        join(&mut session, connection);
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
    send(&mut session, 1, ClientCommand::StartGame);
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    session.game = Some(McGuessGameState::new(
        answer.clone(),
        vec![PlayerId(0), PlayerId(1)],
        rules,
    ));
    let won = send(
        &mut session,
        1,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: answer.id.clone(),
            }
            .into(),
        ),
    );
    let first = snapshot(&won, 1);
    let match_id = first.match_id;
    assert_eq!(
        first
            .scores
            .iter()
            .map(|score| score.score)
            .collect::<Vec<_>>(),
        vec![7, 0]
    );
    assert_eq!(
        first
            .score_gains
            .iter()
            .map(|gain| gain.after - gain.before)
            .collect::<Vec<_>>(),
        vec![7]
    );
    assert!(first.reference_changes.is_empty());
    assert_eq!(session.players[0].completed_games, 0);
    assert!(session.players[0].game_profiles.mc_guess.is_none());
    send(&mut session, 1, ClientCommand::PlayAgain);
    send(&mut session, 2, ClientCommand::PlayAgain);
    session.game = Some(McGuessGameState::new(
        answer.clone(),
        vec![PlayerId(1), PlayerId(0)],
        rules,
    ));
    send(
        &mut session,
        2,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: "未知词条".to_owned(),
            }
            .into(),
        ),
    );
    send(
        &mut session,
        1,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    let won = send(
        &mut session,
        2,
        ClientCommand::Game(McGuessCommand::Guess { entry: answer.id }.into()),
    );
    let last = snapshot(&won, 1);
    assert_eq!(last.match_id, match_id);
    assert_eq!(
        last.scores
            .iter()
            .map(|score| score.score)
            .collect::<Vec<_>>(),
        vec![7, 6]
    );
    assert_eq!(
        last.reference_changes
            .iter()
            .map(|change| change.delta)
            .collect::<Vec<_>>(),
        vec![2, -2]
    );
    assert_eq!(
        (
            session.players[0].reference_points,
            session.players[1].reference_points
        ),
        (2, -2)
    );
    assert_eq!(session.players[0].completed_games, 1);
    let first = last.players[0]
        .player
        .game_profiles
        .mc_guess
        .as_ref()
        .unwrap();
    assert_eq!(
        (
            first.completed_games,
            first.total_score,
            first.total_reference_delta
        ),
        (1, 7, 2)
    );
    assert_eq!(first.placement_counts, [1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        (
            first.small_games(),
            first.wins(),
            first.actions(),
            first.skips
        ),
        (2, 1, 2, 1)
    );
    let second = last.players[1]
        .player
        .game_profiles
        .mc_guess
        .clone()
        .unwrap();
    assert_eq!(
        (
            second.small_games(),
            second.wins(),
            second.unknown_guesses,
            second.unknown_enabled_guesses
        ),
        (2, 1, 1, 2)
    );
    assert_eq!(second.placement_counts, [0, 1, 0, 0, 0, 0, 0, 0]);
    send(&mut session, 1, ClientCommand::PlayAgain);
    assert_eq!(session.players[0].completed_games, 1);
    let start = send(&mut session, 2, ClientCommand::PlayAgain);
    let next = snapshot(&start, 1);
    assert_ne!(next.match_id, match_id);
    assert!(next.scores.iter().all(|score| score.score == 0));
    assert!(next.score_gains.is_empty());
    assert!(next.reference_changes.is_empty());
    assert_eq!(
        next.players[1].player.game_profiles.mc_guess.as_ref(),
        Some(&second)
    );
}
