use super::super::McGuessSession;
use super::support::{join, send, snapshot};
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessMode, McGuessRuleSet,
};
use leocard_protocol::{ClientCommand, McGuessCommand, RoomId, SeatId};
use std::time::Duration;

fn started_race(bot: bool) -> McGuessSession {
    let mut rules = McGuessRuleSet {
        mode: McGuessMode::Race,
        ..Default::default()
    };
    rules.race.seconds = 40;
    let mut session = McGuessSession::new(RoomId(99), 52300, rules);
    join(&mut session, 1);
    send(
        &mut session,
        1,
        ClientCommand::SelectSeat { seat: SeatId(0) },
    );
    if bot {
        send(
            &mut session,
            1,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(1),
                occupied: true,
            },
        );
    } else {
        join(&mut session, 2);
        send(
            &mut session,
            2,
            ClientCommand::SelectSeat { seat: SeatId(1) },
        );
        send(&mut session, 2, ClientCommand::SetReady { ready: true });
    }
    send(&mut session, 1, ClientCommand::SetReady { ready: true });
    send(&mut session, 1, ClientCommand::StartGame);
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:blue_wool")
        .unwrap()
        .clone();
    session.game = Some(McGuessGameState::new(
        answer,
        vec![PlayerId(0), PlayerId(1)],
        rules,
    ));
    session
}

fn guess(session: &mut McGuessSession, connection: u64, id: &str) {
    send(
        session,
        connection,
        ClientCommand::Game(
            McGuessCommand::Guess {
                entry: id.to_owned(),
            }
            .into(),
        ),
    );
}

#[test]
fn submissions_do_not_reset_public_deadline_and_expiry_is_settled_once() {
    let mut session = started_race(false);
    session.advance_time(Duration::from_secs(7));
    guess(&mut session, 2, "minecraft:red_wool");
    guess(&mut session, 2, "minecraft:green_wool");
    assert_eq!(session.guess_seconds_remaining(), Some(33));
    assert_eq!(session.game.as_ref().unwrap().view().guesses.len(), 2);
    let ended = session.advance_time(Duration::from_secs(33));
    let view = snapshot(&ended, 1);
    assert!(view.round.finished());
    assert!(view.round.skipped_players.is_empty());
    assert_eq!(view.round.winner, None);
    assert_eq!(view.guess_seconds_remaining, None);
    assert_eq!(view.score_gains.len(), 1);
    assert_eq!(
        view.scores
            .iter()
            .find(|score| score.player.0 == 1)
            .unwrap()
            .score,
        3
    );
    assert!(session.advance_time(Duration::from_secs(100)).is_empty());
}

#[test]
fn bots_wait_eight_seconds_even_when_people_guess_in_the_meantime() {
    let mut session = started_race(true);
    session.advance_time(Duration::from_secs(7));
    assert!(session.game.as_ref().unwrap().view().guesses.is_empty());
    guess(&mut session, 1, "minecraft:red_wool");
    assert_eq!(session.game.as_ref().unwrap().view().guesses.len(), 1);
    session.advance_time(Duration::from_secs(1));
    let view = session.game.as_ref().unwrap().view();
    assert_eq!(view.guesses.len(), 2);
    assert_eq!(view.guesses[1].player, PlayerId(1));
    assert!(
        view.guesses
            .iter()
            .all(|guess| guess.result.name() != "红色羊毛" || guess.player == PlayerId(0))
    );
}
