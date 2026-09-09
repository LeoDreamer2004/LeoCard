use super::*;

#[test]
fn profile_events_record_uno_penalties_and_challenge_results() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.match_profile_stats = vec![UnoProfileStats::default(); 3];
    session.record_profile_outcome(&ActionOutcome::UnoCalled {
        player: UnoPlayerId(0),
    });
    session.record_profile_outcome(&ActionOutcome::UnoReported {
        reporter: UnoPlayerId(1),
        target: UnoPlayerId(0),
        cards: vec![
            UnoCard::number(UnoColor::Blue, 1, 0),
            UnoCard::number(UnoColor::Blue, 2, 0),
        ],
    });
    session.record_profile_outcome(&ActionOutcome::ChallengeResolved {
        challenger: UnoPlayerId(1),
        offender: UnoPlayerId(2),
        result: UnoChallengeResult::Successful,
        penalized: UnoPlayerId(2),
        cards: vec![UnoCard::number(UnoColor::Green, 3, 0); 4],
        next_player: UnoPlayerId(1),
    });
    session.record_profile_outcome(&ActionOutcome::ChallengeResolved {
        challenger: UnoPlayerId(0),
        offender: UnoPlayerId(2),
        result: UnoChallengeResult::Failed,
        penalized: UnoPlayerId(0),
        cards: vec![UnoCard::number(UnoColor::Yellow, 4, 0); 6],
        next_player: UnoPlayerId(1),
    });

    assert_eq!(session.match_profile_stats[0].uno_calls, 1);
    assert_eq!(session.match_profile_stats[0].uno_penalties, 1);
    assert_eq!(session.match_profile_stats[0].challenges, 1);
    assert_eq!(session.match_profile_stats[0].successful_challenges, 0);
    assert_eq!(session.match_profile_stats[0].max_penalty_cards, 6);
    assert_eq!(session.match_profile_stats[1].challenges, 1);
    assert_eq!(session.match_profile_stats[1].successful_challenges, 1);
    assert_eq!(session.match_profile_stats[2].challenges_received, 2);
    assert_eq!(
        session.match_profile_stats[2].successful_challenges_received,
        1
    );
    assert_eq!(session.match_profile_stats[2].max_penalty_cards, 4);
}

#[test]
fn finished_game_merges_uno_profile_statistics_once() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));
    assert_eq!(session.match_profile_stats[0].max_hand_cards, 7);
    session.match_profile_stats[0].max_hand_cards = 40;
    session.match_profile_stats[0].max_penalty_cards = 12;
    session.match_profile_stats[0].challenges = 3;
    session.match_profile_stats[0].successful_challenges = 2;

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
    assert_eq!(first.max_hand_cards, 40);
    assert!(first.max_penalty_cards >= 12);
    assert_eq!(first.challenges, 3);
    assert_eq!(first.successful_challenges, 2);
}
