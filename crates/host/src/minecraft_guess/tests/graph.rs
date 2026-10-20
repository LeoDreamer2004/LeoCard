use super::super::MinecraftGuessSession;
use super::support::{join, send, snapshot};
use leocard_minecraft_guess::{GraphRules, GuessMode, MinecraftGuessRuleSet};
use leocard_protocol::{
    ClientCommand, GameViolation, MinecraftGuessCommand, RejectReason, RoomId, SeatId, ServerEvent,
};
use std::thread;
use std::time::{Duration, Instant};

pub(super) fn started_session(guess_seconds: u16) -> MinecraftGuessSession {
    let mut session = MinecraftGuessSession::new(
        RoomId(99),
        52300,
        MinecraftGuessRuleSet {
            mode: GuessMode::Graph,
            graph: GraphRules {
                guess_seconds,
                ..Default::default()
            },
            ..Default::default()
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
    let start = send(&mut session, 1, ClientCommand::StartGame);
    assert_eq!(snapshot(&start, 1).graph.as_ref().unwrap().selected, 1);
    assert_eq!(snapshot(&start, 2).graph.as_ref().unwrap().selected, 1);
    for recipient in [1, 2] {
        let pending = snapshot(&start, recipient);
        assert!(pending.graph.as_ref().unwrap().nodes.is_empty());
        assert!(!pending.can_guess());
        assert_eq!(pending.guess_seconds_remaining, None);
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while session.generation.is_some() {
        let updates = session.advance_time(Duration::ZERO);
        if session.generation.is_none() {
            assert!(
                !snapshot(&updates, 1)
                    .graph
                    .as_ref()
                    .unwrap()
                    .nodes
                    .is_empty()
            );
            break;
        }
        assert!(Instant::now() < deadline, "graph generation timed out");
        thread::sleep(Duration::from_millis(1));
    }
    session
}

#[test]
fn developer_activate_all_reveals_the_graph_for_everyone_and_only_finishes_this_round() {
    let mut session = started_session(30);
    let before = session.game.as_ref().unwrap().graph_view().unwrap();
    let result = send(
        &mut session,
        2,
        ClientCommand::Game(MinecraftGuessCommand::DeveloperActivateAll.into()),
    );
    if !(cfg!(debug_assertions) || cfg!(feature = "developer")) {
        assert!(result.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::DeveloperFeatureUnavailable)
            }
        )));
        assert_eq!(session.game.as_ref().unwrap().graph_view().unwrap(), before);
        assert!(!session.game.as_ref().unwrap().finished());
        return;
    }
    for connection in [1, 2] {
        let view = snapshot(&result, connection);
        assert!(view.round.finished());
        assert_eq!(view.round.current_player, None);
        assert_eq!(view.guess_seconds_remaining, None);
        let graph = view.graph.as_ref().unwrap();
        assert!(graph.nodes.iter().all(|node| node.revealed()));
        assert!(
            graph
                .edges
                .iter()
                .all(|edge| edge.active && edge.directed.is_some())
        );
        assert!(graph.guesses.is_empty());
        assert!(view.score_gains.is_empty());
        assert_eq!(view.progress.game_index, 0);
    }
    let repeated = send(
        &mut session,
        2,
        ClientCommand::Game(MinecraftGuessCommand::DeveloperActivateAll.into()),
    );
    assert!(
        repeated
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    send(&mut session, 1, ClientCommand::PlayAgain);
    let next = send(&mut session, 2, ClientCommand::PlayAgain);
    assert_eq!(snapshot(&next, 1).progress.game_index, 1);
    assert!(!snapshot(&next, 1).round.finished());
}

#[test]
fn selecting_nodes_broadcasts_the_same_history_without_advancing_turns() {
    let mut session = started_session(60);
    let before = session.game.as_ref().unwrap().view().current_player;
    let selected = send(
        &mut session,
        1,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number: 2 }.into()),
    );
    for connection in [1, 2] {
        let view = snapshot(&selected, connection);
        assert_eq!(view.graph.as_ref().unwrap().selected, 2);
        assert_eq!(view.round.current_player, before);
        assert!(view.round.guesses.is_empty());
        assert!(view.score_gains.is_empty());
    }
    send(
        &mut session,
        1,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number: 3 }.into()),
    );
    assert_eq!(
        session
            .game
            .as_ref()
            .unwrap()
            .graph_view()
            .unwrap()
            .selected,
        2
    );
    session.advance_time(Duration::from_millis(600));
    send(
        &mut session,
        2,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number: 3 }.into()),
    );
    assert_eq!(
        session
            .game
            .as_ref()
            .unwrap()
            .graph_view()
            .unwrap()
            .selected,
        2
    );
    let selected = send(
        &mut session,
        1,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number: 3 }.into()),
    );
    assert_eq!(snapshot(&selected, 1).graph, snapshot(&selected, 2).graph);
    assert_eq!(snapshot(&selected, 1).graph.as_ref().unwrap().selected, 3);
}

#[test]
fn graph_turn_clock_expires_without_revealing_nodes_and_switches_do_not_reset_it() {
    let mut session = started_session(30);
    let tick = session.advance_time(Duration::from_secs(5));
    assert_eq!(snapshot(&tick, 1).guess_seconds_remaining, Some(25));
    let switched = send(
        &mut session,
        1,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number: 2 }.into()),
    );
    assert_eq!(snapshot(&switched, 1).guess_seconds_remaining, Some(25));
    let expired = session.advance_time(Duration::from_secs(25));
    let game = snapshot(&expired, 1);
    assert_eq!(game.round.current_player, Some(1));
    assert_eq!(game.guess_seconds_remaining, Some(30));
    assert_eq!(game.graph.as_ref().unwrap().selected, 2);
    let expired_again = session.advance_time(Duration::from_secs(30));
    let game = snapshot(&expired_again, 1);
    assert_eq!(game.round.current_player, Some(0));
    assert_eq!(game.guess_seconds_remaining, Some(30));
    assert!(!game.round.finished());
    assert!(game.round.guesses.is_empty());
    assert!(
        game.graph
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .all(|node| !node.revealed())
    );

    let mut unlimited = started_session(0);
    assert!(unlimited.advance_time(Duration::from_secs(600)).is_empty());
    assert_eq!(unlimited.guess_seconds_remaining(), None);
    assert_eq!(unlimited.game.as_ref().unwrap().current_player(), Some(0));
}
