use super::{command, join, send};
use crate::ConnectionId;
use crate::draw_guess::DrawGuessSession;
use crate::features::GameSnapshotView;
use leocard_draw_guess::DrawGuessRuleSet;
use leocard_protocol::{
    ClientCommand, DrawGuessCommand, GameSnapshot, RoomId, ServerEvent, ServerMessage,
    decode_frame, encode_frame,
};
use std::time::Duration;

#[test]
fn post_solve_chat_never_reaches_unguessed_players_or_spectators() {
    let mut session =
        DrawGuessSession::new(RoomId(99), 52300, DrawGuessRuleSet::default()).unwrap();
    for connection in 1..=3 {
        join(&mut session, connection);
    }
    send(&mut session, 1, ClientCommand::StartGame);
    command(
        &mut session,
        1,
        DrawGuessCommand::ChooseWord { turn: 0, choice: 0 },
    );
    let artist = session.order[0];
    let solved = session.order[1];
    let word = session.game_view(artist).state.word.unwrap();
    command(
        &mut session,
        2,
        DrawGuessCommand::Guess {
            turn: 0,
            text: word,
        },
    );
    join(&mut session, 4);
    send(
        &mut session,
        4,
        ClientCommand::SpectatePlayer { player: solved },
    );
    session.advance_time(Duration::from_millis(350));
    let replies = command(
        &mut session,
        2,
        DrawGuessCommand::Guess {
            turn: 0,
            text: "只给已经猜中的玩家看".into(),
        },
    );
    let snapshots: Vec<_> = replies
        .into_iter()
        .filter(|delivery| {
            matches!(
                delivery.message.event,
                ServerEvent::GameSnapshot(GameSnapshot::DrawGuess(_))
            )
        })
        .collect();
    assert_eq!(snapshots.len(), 4);
    for delivery in snapshots {
        let message =
            decode_frame::<ServerMessage>(&encode_frame(&delivery.message).unwrap()).unwrap();
        let ServerEvent::GameSnapshot(GameSnapshot::DrawGuess(view)) = message.event else {
            panic!("expected drawing snapshot");
        };
        let private = view.state.history.iter().find(|message| message.private);
        assert_eq!(
            private.is_some(),
            matches!(delivery.recipient, ConnectionId(1 | 2)),
        );
        if let Some(message) = private {
            assert_eq!(message.text, "只给已经猜中的玩家看");
            assert!(!message.correct);
            assert_eq!(message.points, 0);
        }
    }
}
