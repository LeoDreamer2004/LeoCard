use super::super::MinecraftGuessSession;
use super::graph::started_session;
use super::support::{send, snapshot};
use crate::Delivery;
use leocard_minecraft_guess::{GameError, GameState, GraphView};
use leocard_protocol::{
    ClientCommand, GameViolation, MinecraftGuessCommand, RejectReason, ServerEvent,
};
use std::time::Duration;

fn session() -> (MinecraftGuessSession, Vec<Option<String>>) {
    let mut session = started_session(30);
    let game = GameState::new_graph(vec![0, 1], session.rules, 5).unwrap();
    let mut revealed = game.clone();
    revealed.reveal_graph().unwrap();
    let answers = revealed
        .graph_view()
        .unwrap()
        .nodes
        .into_iter()
        .map(|node| node.entry.map(|entry| entry.id))
        .collect();
    session.game = Some(game);
    (session, answers)
}

fn guess(session: &mut MinecraftGuessSession, connection: u64, answer: &str) -> Vec<Delivery> {
    send(
        session,
        connection,
        ClientCommand::Game(
            MinecraftGuessCommand::Guess {
                entry: answer.to_owned(),
            }
            .into(),
        ),
    )
}

fn select(session: &mut MinecraftGuessSession, connection: u64, number: u16) -> Vec<Delivery> {
    send(
        session,
        connection,
        ClientCommand::Game(MinecraftGuessCommand::SelectNode { number }.into()),
    )
}

fn target(graph: &GraphView) -> u16 {
    graph.combo.as_ref().unwrap().targets[0]
}

#[test]
fn host_preserves_combo_turns_broadcasts_counts_and_rejects_distant_targets() {
    let (mut session, answers) = session();
    let deliveries = guess(&mut session, 1, answers[0].as_deref().unwrap());
    for connection in [1, 2] {
        let view = snapshot(&deliveries, connection);
        assert_eq!(view.round.current_player, Some(0));
        assert_eq!(view.guess_seconds_remaining, Some(30));
        let graph = view.graph.as_ref().unwrap();
        assert_eq!(graph.combo.as_ref().unwrap().count, 1);
        assert_eq!(graph.guesses.last().unwrap().combo, 1);
    }
    assert_eq!(
        snapshot(&deliveries, 1).graph,
        snapshot(&deliveries, 2).graph
    );
    let first_score = snapshot(&deliveries, 1)
        .scores
        .iter()
        .find(|score| score.player.0 == 0)
        .unwrap()
        .score;
    assert!(first_score > 0);
    let before = session.game.as_ref().unwrap().graph_view().unwrap();
    let distant = before
        .nodes
        .iter()
        .find(|node| node.kind.is_some() && !node.revealed() && !before.can_guess_node(node.number))
        .unwrap()
        .number;
    session.advance_time(Duration::from_secs(3));
    let rejected = select(&mut session, 1, distant);
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::MinecraftGuess(
                GameError::ComboRequiresNeighbour
            ))
        }
    )));
    assert_eq!(session.game.as_ref().unwrap().graph_view().unwrap(), before);
    assert_eq!(session.guess_seconds_remaining(), Some(27));

    let next = target(&before);
    select(&mut session, 1, next);
    let deliveries = guess(
        &mut session,
        1,
        answers[usize::from(next - 1)].as_deref().unwrap(),
    );
    for connection in [1, 2] {
        let view = snapshot(&deliveries, connection);
        assert_eq!(view.round.current_player, Some(0));
        assert_eq!(view.guess_seconds_remaining, Some(30));
        let graph = view.graph.as_ref().unwrap();
        assert_eq!(graph.guesses.last().unwrap().combo, 2);
        assert_eq!(graph.combo.as_ref().unwrap().count, 2);
        let score = view
            .scores
            .iter()
            .find(|score| score.player.0 == 0)
            .unwrap();
        assert!(score.score > first_score);
        let gain = view
            .score_gains
            .iter()
            .rev()
            .find(|gain| gain.player == 0)
            .unwrap();
        assert_eq!(gain.before, first_score);
        assert_eq!(gain.after, score.score);
    }

    let expired = session.advance_time(Duration::from_secs(30));
    let view = snapshot(&expired, 1);
    assert_eq!(view.round.current_player, Some(1));
    assert!(view.graph.as_ref().unwrap().combo.is_none());
    let next = view
        .graph
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.kind.is_some() && !node.revealed())
        .unwrap()
        .number;
    select(&mut session, 2, next);
    let deliveries = guess(
        &mut session,
        2,
        answers[usize::from(next - 1)].as_deref().unwrap(),
    );
    assert_eq!(
        snapshot(&deliveries, 2)
            .graph
            .as_ref()
            .unwrap()
            .guesses
            .last()
            .unwrap()
            .combo,
        1
    );
    let skipped = send(
        &mut session,
        2,
        ClientCommand::Game(MinecraftGuessCommand::Skip.into()),
    );
    assert!(
        snapshot(&skipped, 1)
            .graph
            .as_ref()
            .unwrap()
            .combo
            .is_none()
    );
    assert_eq!(snapshot(&skipped, 1).round.current_player, Some(0));
}

#[test]
fn automated_players_select_a_legal_neighbour_after_a_correct_guess() {
    let (mut session, answers) = session();
    guess(&mut session, 1, answers[0].as_deref().unwrap());
    let before = session.game.as_ref().unwrap().graph_view().unwrap();
    session.room.players[0].is_bot = true;
    let deliveries = session.advance_time(Duration::from_secs(2));
    let graph = snapshot(&deliveries, 2).graph.as_ref().unwrap();
    assert_eq!(graph.guesses.len(), before.guesses.len() + 1);
    let last = graph.guesses.last().unwrap();
    assert_eq!(last.guess.player, 0);
    assert!(before.combo.as_ref().unwrap().targets.contains(&last.node));
}
