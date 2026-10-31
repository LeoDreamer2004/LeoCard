use super::graph::started_session_with_graph;
use super::support::send;
use leocard_mc_guess::McGuessGraphRules;
use leocard_protocol::{ClientCommand, GameEvent, McGuessCommand, McGuessEvent, ServerEvent};

#[test]
fn completed_graph_merges_node_statistics_into_existing_archives_once() {
    let mut session = started_session_with_graph(McGuessGraphRules {
        round_robin: false,
        ..Default::default()
    });
    let mut revealed = session.game.clone().unwrap();
    revealed.reveal_graph().unwrap();
    let answers = revealed.graph_view().unwrap();
    let target = answers
        .nodes
        .iter()
        .find(|node| node.entry.is_some())
        .unwrap();
    let answer = target.entry.as_ref().unwrap();
    let player = session
        .game
        .as_ref()
        .unwrap()
        .view()
        .current_player
        .unwrap();
    let connection = u64::from(player.0) + 1;
    let previous = session
        .room
        .players
        .iter_mut()
        .find(|record| record.id == player)
        .unwrap()
        .game_profiles
        .mc_guess
        .get_or_insert_with(Default::default);
    previous.categories.block.games = 12;
    previous.categories.block.wins = 3;
    previous.known_guesses = 20;
    previous.completed_games = 4;
    let old = previous.clone();
    send(
        &mut session,
        connection,
        ClientCommand::Game(
            McGuessCommand::SelectNode {
                number: target.number,
            }
            .into(),
        ),
    );
    let deliveries = send(
        &mut session,
        connection,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: answer.id.clone(),
            }
            .into(),
        ),
    );
    assert!(deliveries.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::ActionAnalyzed { player: actor, statistics }))
            if *actor == player && statistics.correct_entry.as_ref().is_some_and(|entry| entry.id == answer.id)
    )));
    for delivery in &deliveries {
        if let ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::HandAnalyzed {
            player,
            ..
        })) = &delivery.message.event
        {
            assert_eq!(session.room.player_id(delivery.recipient), Some(*player));
        }
    }
    assert!(deliveries.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::HandAnalyzed { player: actor, statistics }))
            if *actor == player && statistics.consistent
    )));
    // Finish the fixture without relying on the debug-only activation command.
    session.game.as_mut().unwrap().reveal_graph().unwrap();
    session.after_action();
    let archive = session
        .room
        .players
        .iter()
        .find(|record| record.id == player)
        .unwrap()
        .game_profiles
        .mc_guess
        .as_ref()
        .unwrap();
    assert_eq!(archive.completed_games, old.completed_games + 1);
    assert_eq!(archive.known_guesses, old.known_guesses + 1);
    assert_eq!(archive.wins(), old.wins() + 1);
    assert_eq!(
        archive.small_games(),
        old.small_games()
            + answers
                .nodes
                .iter()
                .filter(|node| node.entry.is_some())
                .count() as u32
    );
    assert_eq!(
        archive.categories.get(answer.kind).wins,
        old.categories.get(answer.kind).wins + 1
    );
    let settled = archive.clone();
    session.settle_match();
    assert_eq!(
        session
            .room
            .players
            .iter()
            .find(|record| record.id == player)
            .unwrap()
            .game_profiles
            .mc_guess
            .as_ref(),
        Some(&settled)
    );
}
