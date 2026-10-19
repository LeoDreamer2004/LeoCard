use super::super::MinecraftGuessSession;
use super::support::{join, send, snapshot};
use leocard_minecraft_guess::{GuessMode, MinecraftGuessRuleSet};
use leocard_protocol::{ClientCommand, MinecraftGuessCommand, RoomId, SeatId};
use std::time::Duration;

#[test]
fn selecting_nodes_broadcasts_the_same_history_without_advancing_turns() {
    let mut session = MinecraftGuessSession::new(
        RoomId(99),
        52300,
        MinecraftGuessRuleSet {
            mode: GuessMode::Graph,
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
