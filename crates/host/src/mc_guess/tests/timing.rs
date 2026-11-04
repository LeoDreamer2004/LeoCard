use super::super::McGuessSession;
use super::graph;
use super::support::{join, send, snapshot};
use crate::ConnectionId;
use leocard_game_common::PlayerId;
use leocard_mc_guess::{McGuessRuleSet, McGuessTurnBasedRules};
use leocard_protocol::{
    ClientCommand, GameEvent, McGuessCommand, McGuessEvent, RoomId, SeatId, ServerEvent,
};
use std::time::Duration;

fn started_session(guess_seconds: u16) -> McGuessSession {
    let mut session = McGuessSession::new(
        RoomId(99),
        52300,
        McGuessRuleSet {
            turn_based: McGuessTurnBasedRules {
                guess_seconds,
                ..McGuessRuleSet::default().turn_based
            },
            ..McGuessRuleSet::default()
        },
    );
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
    session
}

#[test]
fn timeout_skips_once_resets_next_turn_and_stops_after_all_skip() {
    let mut session = started_session(15);
    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    let tick = session.advance_time(Duration::from_millis(1));
    assert_eq!(snapshot(&tick, 1).guess_seconds_remaining, Some(14));
    let timed_out = session.advance_time(Duration::from_secs(14));
    let game = snapshot(&timed_out, 1);
    assert_eq!(game.round.current_player, Some(PlayerId(1)));
    assert_eq!(game.round.skipped_players, vec![PlayerId(0)]);
    assert_eq!(game.guess_seconds_remaining, Some(15));
    // A long host tick still applies only to the player whose turn was active.
    let finished = session.advance_time(Duration::from_secs(60));
    let game = snapshot(&finished, 1);
    assert!(game.round.finished());
    assert_eq!(game.guess_seconds_remaining, None);
    assert!(session.advance_time(Duration::from_secs(60)).is_empty());
}

#[test]
fn rejected_guess_preserves_clock_and_manual_skip_resets_it() {
    let mut session = started_session(30);
    session.advance_time(Duration::from_secs(5));
    send(
        &mut session,
        1,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: String::new(),
            }
            .into(),
        ),
    );
    assert_eq!(session.guess_seconds_remaining(), Some(25));
    let skipped = send(
        &mut session,
        1,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    assert_eq!(snapshot(&skipped, 1).guess_seconds_remaining, Some(30));
}

#[test]
fn unlimited_turns_do_not_expire() {
    let mut session = started_session(0);
    assert_eq!(session.guess_seconds_remaining(), None);
    assert!(session.advance_time(Duration::from_secs(600)).is_empty());
    assert_eq!(
        session.game.as_ref().unwrap().current_player(),
        Some(PlayerId(0))
    );
}

#[test]
fn graph_manual_and_final_timeout_skips_are_broadcast_even_on_tag_nodes() {
    let mut session = graph::started_session(30);
    let tag = session
        .game
        .as_ref()
        .unwrap()
        .graph_view()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.kind.is_none())
        .unwrap()
        .number;
    send(
        &mut session,
        1,
        ClientCommand::Game(McGuessCommand::SelectNode { number: tag }.into()),
    );
    let manual = send(
        &mut session,
        1,
        ClientCommand::Game(McGuessCommand::Skip.into()),
    );
    let finished = session.advance_time(Duration::from_secs(30));
    assert!(snapshot(&finished, 1).round.finished());
    assert!(snapshot(&finished, 1).round.skipped_players.is_empty());
    for (deliveries, actor) in [(&manual, PlayerId(0)), (&finished, PlayerId(1))] {
        for recipient in [ConnectionId(1), ConnectionId(2)] {
            let notices: Vec<_> = deliveries.iter().filter(|delivery| {
                delivery.recipient == recipient && matches!(
                    delivery.message.event,
                    ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::GuessSkipped { player })) if player == actor
                )
            }).collect();
            assert_eq!(notices.len(), 1);
            assert!(notices[0].message.game_context.is_some());
        }
    }
    assert!(session.advance_time(Duration::from_secs(30)).is_empty());
}
