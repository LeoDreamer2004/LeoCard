use super::super::MinecraftGuessSession;
use super::support::{join, send, snapshot};
use leocard_minecraft_guess::{GameState, GraphRules, GuessMode, MinecraftGuessRuleSet};
use leocard_protocol::{ClientCommand, PlayerId, RoomId, SeatId};

fn start(round_robin: bool) -> MinecraftGuessSession {
    let mut session = MinecraftGuessSession::new(
        RoomId(99),
        52300,
        MinecraftGuessRuleSet {
            mode: GuessMode::Graph,
            graph: GraphRules {
                round_robin,
                nodes: 24,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    for connection in 1..=4 {
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
    fastrand::seed(42);
    send(&mut session, 1, ClientCommand::StartGame);
    let order = session.match_state.as_ref().unwrap().turn_order();
    session.game = Some(GameState::new_graph(order, session.rules, 5).unwrap());
    session.generation = None;
    session
}

#[test]
fn single_graph_shuffles_visible_seats_and_the_authoritative_turn_order_together() {
    let mut session = start(false);
    let mut expected = vec![0, 1, 2, 3];
    fastrand::Rng::with_seed(42).shuffle(&mut expected);
    assert_ne!(expected, vec![0, 1, 2, 3]);
    let state = session.match_state.as_ref().unwrap();
    assert_eq!(state.turn_order(), expected);
    assert_eq!(state.progress().total_games(), 1);
    assert!(state.progress().is_last_game());
    let updates = send(&mut session, 1, ClientCommand::RequestSnapshot);
    for connection in 1..=4 {
        let deliveries = send(&mut session, connection, ClientCommand::RequestSnapshot);
        let view = snapshot(&deliveries, connection);
        assert_eq!(
            view.players
                .iter()
                .map(|player| player.id.0)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(view.round.current_player, Some(expected[0]));
        assert_eq!(view.you, PlayerId(connection as u8 - 1));
    }
    let match_id = snapshot(&updates, 1).match_id;
    session.game.as_mut().unwrap().reveal_graph().unwrap();
    session.settle_match();
    for connection in 1..=4 {
        send(&mut session, connection, ClientCommand::PlayAgain);
    }
    assert_ne!(session.match_id.unwrap(), match_id);
    assert_eq!(
        session.match_state.as_ref().unwrap().progress().game_index,
        0
    );
}

#[test]
fn rotation_finishes_after_every_opener_and_settles_exactly_three_times_single_graph() {
    for round_robin in [false, true] {
        let mut session = start(round_robin);
        session
            .match_state
            .as_mut()
            .unwrap()
            .record_gains([(0, 40), (1, 30), (2, 20), (3, 10)]);
        session.game.as_mut().unwrap().reveal_graph().unwrap();
        if round_robin {
            for opener in 0..4 {
                let state = session.match_state.as_ref().unwrap();
                assert_eq!(state.turn_order()[0], opener);
                assert_eq!(state.progress().total_games(), 4);
                session.settle_match();
                if opener < 3 {
                    assert!(session.finished_reference_changes.is_none());
                    assert!(session.match_state.as_mut().unwrap().advance());
                }
            }
        } else {
            session.settle_match();
        }
        assert!(!session.match_state.as_mut().unwrap().advance());
        let changes = session.finished_reference_changes.as_ref().unwrap();
        let multiplier = if round_robin { 3 } else { 1 };
        // 3 * 24 / 16 truncates to 4 before tripling, so a full round awards 12, not 13.
        assert_eq!(changes.iter().map(|change| change.delta).sum::<i16>(), 0);
        for (index, single_delta) in [4, 3, -3, -4].into_iter().enumerate() {
            assert_eq!(
                changes
                    .iter()
                    .find(|change| change.player == PlayerId(index as u8))
                    .unwrap()
                    .delta,
                single_delta * multiplier
            );
            assert_eq!(session.room.players[index].completed_games, 1);
        }
        session.settle_match();
        assert!(
            session
                .room
                .players
                .iter()
                .all(|player| player.completed_games == 1)
        );
    }
}
