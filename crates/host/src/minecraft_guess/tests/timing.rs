use super::super::MinecraftGuessSession;
use super::support::{join, send, snapshot};
use leocard_minecraft_guess::MinecraftGuessRuleSet;
use leocard_minecraft_guess::TurnBasedRules;
use leocard_protocol::{ClientCommand, MinecraftGuessCommand, RoomId, SeatId};
use std::time::Duration;

fn started_session(guess_seconds: u16) -> MinecraftGuessSession {
    let mut session = MinecraftGuessSession::new(
        RoomId(99),
        52300,
        MinecraftGuessRuleSet {
            turn_based: TurnBasedRules {
                guess_seconds,
                ..MinecraftGuessRuleSet::default().turn_based
            },
            ..MinecraftGuessRuleSet::default()
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
    assert_eq!(game.round.current_player, Some(1));
    assert_eq!(game.round.skipped_players, vec![0]);
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
            MinecraftGuessCommand::Guess {
                entry: String::new(),
            }
            .into(),
        ),
    );
    assert_eq!(session.guess_seconds_remaining(), Some(25));
    let skipped = send(
        &mut session,
        1,
        ClientCommand::Game(MinecraftGuessCommand::Skip.into()),
    );
    assert_eq!(snapshot(&skipped, 1).guess_seconds_remaining, Some(30));
}

#[test]
fn unlimited_turns_do_not_expire() {
    let mut session = started_session(0);
    assert_eq!(session.guess_seconds_remaining(), None);
    assert!(session.advance_time(Duration::from_secs(600)).is_empty());
    assert_eq!(session.game.as_ref().unwrap().current_player(), Some(0));
}
