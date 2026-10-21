use super::graph::started_session_with_graph;
use super::support::send;
use leocard_minecraft_guess::GraphRules;
use leocard_protocol::{ClientCommand, MinecraftGuessCommand};

#[test]
fn completed_graph_merges_node_statistics_into_existing_archives_once() {
    let mut session = started_session_with_graph(GraphRules {
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
    let connection = u64::from(player) + 1;
    let previous = session
        .room
        .players
        .iter_mut()
        .find(|record| record.id.0 == player)
        .unwrap()
        .game_profiles
        .minecraft_guess
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
            MinecraftGuessCommand::SelectNode {
                number: target.number,
            }
            .into(),
        ),
    );
    send(
        &mut session,
        connection,
        ClientCommand::Game(
            MinecraftGuessCommand::Guess {
                entry: answer.id.clone(),
            }
            .into(),
        ),
    );
    send(
        &mut session,
        connection,
        ClientCommand::Game(MinecraftGuessCommand::DeveloperActivateAll.into()),
    );
    let archive = session
        .room
        .players
        .iter()
        .find(|record| record.id.0 == player)
        .unwrap()
        .game_profiles
        .minecraft_guess
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
            .find(|record| record.id.0 == player)
            .unwrap()
            .game_profiles
            .minecraft_guess
            .as_ref(),
        Some(&settled)
    );
}
