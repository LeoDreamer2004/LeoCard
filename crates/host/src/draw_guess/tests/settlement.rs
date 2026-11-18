use super::{DrawGuessSession, command, join, send};
use crate::features::GameSnapshotView;
use leocard_draw_guess::{DrawGuessPhase, DrawGuessRuleSet};
use leocard_protocol::{ClientCommand, DrawGuessCommand, RoomId};
use std::time::Duration;

fn started(rounds: u8) -> DrawGuessSession {
    let mut session = DrawGuessSession::new(
        RoomId(99),
        52300,
        DrawGuessRuleSet {
            rounds,
            drawing_seconds: 30,
        },
    )
    .unwrap();
    for connection in 1..=4 {
        join(&mut session, connection);
    }
    send(&mut session, 1, ClientCommand::StartGame);
    session
}

#[test]
fn rank_and_round_awards_settle_once_and_reset_for_a_rematch() {
    for rounds in [1, 3] {
        let mut session = started(rounds);
        for turn in 0..u32::from(rounds) * 4 {
            let artist = turn % 4;
            command(
                &mut session,
                u64::from(artist + 1),
                DrawGuessCommand::ChooseWord { turn, choice: 0 },
            );
            let answer = session
                .game_view(session.order[artist as usize])
                .state
                .word
                .unwrap();
            let mut elapsed = 0;
            for guesser in 0..4 {
                if guesser == artist {
                    continue;
                }
                let next = guesser * 1000;
                session.advance_time(Duration::from_millis(u64::from(next - elapsed)));
                elapsed = next;
                command(
                    &mut session,
                    u64::from(guesser + 1),
                    DrawGuessCommand::Guess {
                        turn,
                        text: answer.clone(),
                    },
                );
            }
            assert_eq!(
                session.game.as_ref().unwrap().phase(),
                DrawGuessPhase::Reveal
            );
            assert!(
                session
                    .game_view(session.order[0])
                    .reference_changes
                    .is_empty()
            );
            session.advance_time(Duration::from_secs(5));
        }
        let finished = session.game_view(session.order[0]);
        assert!(finished.match_complete);
        let multiplier = if rounds >= 3 { 2 } else { 1 };
        assert_eq!(
            finished
                .reference_changes
                .iter()
                .map(|change| change.delta)
                .collect::<Vec<_>>(),
            [3, 2, -2, -3].map(|delta| delta * multiplier)
        );
        for (player, change) in session.room.players.iter().zip(&finished.reference_changes) {
            assert_eq!(player.id, change.player);
            assert_eq!(player.reference_points, i32::from(change.delta));
            assert_eq!(player.completed_games, 1);
        }
        session.advance_time(Duration::from_secs(60));
        for connection in 1..=4 {
            send(&mut session, connection, ClientCommand::PlayAgain);
            assert!(
                session
                    .room
                    .players
                    .iter()
                    .all(|player| player.completed_games == 1)
            );
            if connection < 4 {
                assert_eq!(
                    session.game_view(session.order[0]).reference_changes,
                    finished.reference_changes
                );
            }
        }
        let next = session.game_view(session.order[0]);
        assert_ne!(next.match_id, finished.match_id);
        assert!(!next.match_complete);
        assert!(next.reference_changes.is_empty());
        assert!(next.state.players.iter().all(|player| player.score == 0));
    }
}

#[test]
fn timeout_ties_share_the_highest_award_and_lobby_clears_the_settlement() {
    let mut session = started(1);
    session.advance_time(Duration::from_secs(4 * 50));
    let finished = session.game_view(session.order[0]);
    assert!(finished.match_complete);
    assert!(
        finished
            .state
            .players
            .iter()
            .all(|player| player.score == 0)
    );
    assert_eq!(
        finished
            .reference_changes
            .iter()
            .map(|change| change.delta)
            .collect::<Vec<_>>(),
        [3; 4]
    );
    send(&mut session, 1, ClientCommand::ReturnToLobby);
    assert!(session.game.is_none());
    assert!(session.finished_reference_changes.is_none());
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.reference_points == 3 && player.completed_games == 1)
    );
}
