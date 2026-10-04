use super::super::*;
use super::*;
use crate::ConnectionId;
use leocard_protocol::ClientCommand;
#[cfg(test)]
use leocard_uno::build_deck;
use leocard_uno::{Phase, UnoRuleSet};

#[test]
fn finished_game_merges_uno_profile_statistics_once() {
    let mut session = UnoSession::new(ROOM, 52300, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    assert_eq!(statistics(&session, 0).peak_hand, 7);

    for _ in 0..5_000 {
        if session
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            break;
        }
        session
            .play_automatic_action()
            .expect("an automatic UNO action should remain available");
    }
    let result = match session.game.as_ref().unwrap().phase() {
        Phase::Finished(result) => result.clone(),
        Phase::Playing => panic!("automatic play did not finish the game"),
    };

    session.apply_finished_reference_points();
    session.apply_finished_reference_points();

    for (index, participant) in session.room.players.iter().enumerate() {
        let stats = participant.game_profiles.uno.as_ref().unwrap();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(
            stats.total_reference_delta,
            i64::from(result.reference_deltas[index])
        );
        assert_eq!(
            stats.total_remaining_score,
            u64::from(result.hand_scores[index])
        );
        assert_eq!(stats.placement_counts.iter().sum::<u32>(), 1);
    }
    let first = session.room.players[0].game_profiles.uno.as_ref().unwrap();
    let recorded = statistics(&session, 0);
    assert_eq!(first.max_hand_cards, recorded.peak_hand);
    assert_eq!(first.max_penalty_cards, recorded.max_penalty_cards);
    assert_eq!(first.uno_calls, recorded.uno_calls);
}
