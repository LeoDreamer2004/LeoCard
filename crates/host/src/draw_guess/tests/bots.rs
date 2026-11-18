use super::{command, join, send};
use crate::draw_guess::DrawGuessSession;
use crate::features::GameSnapshotView;
use leocard_draw_guess::{DrawGuessPhase, DrawGuessRuleSet};
use leocard_protocol::{ClientCommand, DrawGuessCommand, RoomId, SeatId, ServerEvent};
use std::time::Duration;

fn with_bots(humans: u8, bots: u8) -> DrawGuessSession {
    let mut session = DrawGuessSession::new(
        RoomId(99),
        52300,
        DrawGuessRuleSet {
            rounds: 1,
            drawing_seconds: 30,
        },
    )
    .unwrap();
    for connection in 1..=humans {
        join(&mut session, u64::from(connection));
    }
    for seat in humans..humans + bots {
        let deliveries = send(
            &mut session,
            1,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(seat),
                occupied: true,
            },
        );
        assert!(
            deliveries
                .iter()
                .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
    }
    send(&mut session, 1, ClientCommand::StartGame);
    assert_eq!(session.order.len(), usize::from(humans + bots));
    assert!(
        session
            .game_view(session.order[0])
            .players
            .iter()
            .all(|player| player.connected)
    );
    command(
        &mut session,
        1,
        DrawGuessCommand::ChooseWord { turn: 0, choice: 0 },
    );
    session
}

#[test]
fn bots_guess_at_five_seconds_without_repeating_or_exposing_the_answer() {
    let mut session = with_bots(2, 2);
    let guesser = session.order[1];
    session.advance_time(Duration::from_millis(24_999));
    let before = session.game_view(guesser).state;
    assert!(before.history.is_empty());
    assert!(before.players.iter().all(|player| !player.guessed));

    let deliveries = session.advance_time(Duration::from_millis(1));
    assert!(!deliveries.is_empty());
    let guessed = session.game_view(guesser).state;
    assert_eq!(guessed.remaining_ms, 5000);
    assert_eq!(guessed.phase, DrawGuessPhase::Drawing);
    assert!(guessed.word.is_none());
    assert_eq!(guessed.history.len(), 2);
    assert!(
        guessed
            .history
            .iter()
            .all(|message| message.correct && message.text.is_empty() && message.points == 33)
    );
    assert!(guessed.players[2..].iter().all(|player| player.guessed));
    session.advance_time(Duration::from_secs(4));
    assert_eq!(session.game_view(guesser).state.history, guessed.history);
}

#[test]
fn bot_artist_leaves_a_blank_canvas_and_is_ready_for_the_next_match() {
    let mut session = with_bots(1, 1);
    let human = session.order[0];
    let bot = session.order[1];
    let first_match = session.match_id;
    session.advance_time(Duration::from_secs(25));
    assert_eq!(session.game_view(human).state.phase, DrawGuessPhase::Reveal);
    session.advance_time(Duration::from_secs(5));
    let choosing = session.game_view(human).state;
    assert_eq!(choosing.phase, DrawGuessPhase::Choosing);
    assert_eq!(choosing.artist, Some(bot));
    session.advance_time(Duration::from_secs(15));
    assert_eq!(
        session.game_view(human).state.phase,
        DrawGuessPhase::Drawing
    );
    session.advance_time(Duration::from_millis(29_999));
    let drawing = session.game_view(human).state;
    assert_eq!(drawing.phase, DrawGuessPhase::Drawing);
    assert!(drawing.canvas.operations().is_empty());
    assert!(drawing.canvas.pending().is_none());
    assert!(drawing.history.is_empty());
    session.advance_time(Duration::from_millis(5001));
    assert!(session.game_view(human).match_complete);
    assert!(
        session
            .room
            .players
            .iter()
            .find(|player| player.id == bot)
            .unwrap()
            .ready
    );
    send(&mut session, 1, ClientCommand::PlayAgain);
    assert_ne!(session.match_id, first_match);
    let rematch = session.game_view(human);
    assert_eq!(rematch.players.len(), 2);
    assert_eq!(rematch.state.phase, DrawGuessPhase::Choosing);
    assert!(rematch.state.players.iter().all(|player| player.score == 0));
}

#[test]
fn large_time_steps_keep_the_same_bot_guesses_and_scores() {
    let mut session = with_bots(1, 7);
    let mut small_steps = session.clone();
    session.advance_time(Duration::from_secs(400));
    for _ in 0..400 {
        small_steps.advance_time(Duration::from_secs(1));
    }
    let human = session.order[0];
    let finished = session.game_view(human);
    assert!(finished.match_complete);
    assert_eq!(finished.state, small_steps.game_view(human).state);
    assert_eq!(finished.state.players[0].score, 350);
    assert!(
        finished.state.players[1..]
            .iter()
            .all(|player| player.score == 531)
    );
}
