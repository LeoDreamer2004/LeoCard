use super::*;
use std::time::Duration;

#[test]
fn tribute_return_and_play_are_separate_private_safe_human_decisions() {
    let mut session = started();
    let rules = session.rules;
    let previous = GuandanGameResult {
        finishing_order: [PlayerId(0), PlayerId(2), PlayerId(1), PlayerId(3)],
        winning_team: 0,
        level_gain: 3,
        known_positions: 2,
    };
    // The ordered deck splits the big jokers, avoiding tribute resistance.
    session.game = Some(
        GuandanGameState::new_with_tribute(rules, build_deck(), PlayerId(0), Some(previous))
            .unwrap(),
    );
    send(&mut session, 1, ClientCommand::RequestSnapshot);
    for index in 0..5 {
        let game = session.game.as_ref().unwrap();
        assert_eq!(
            game.phase(),
            if index < 4 {
                &GuandanPhase::Tribute
            } else {
                &GuandanPhase::Playing
            }
        );
        let actor = game.current_player().unwrap();
        let action = auto_play_action(game, actor).unwrap();
        let before = session.room.players[actor.index()]
            .game_profiles
            .guandan
            .as_ref()
            .map(|stats| stats.action_time)
            .unwrap_or_default();
        session.advance_time(Duration::from_secs(61));
        let public = session.room.action_timing.snapshot().unwrap();
        assert_eq!(public.clocks.len(), 1);
        assert_eq!(
            (public.clocks[0].player, public.clocks[0].elapsed_millis),
            (actor, 61_000)
        );
        // The recipient-only tribute action/received card never enters the public clock DTO.
        let deliveries = send(
            &mut session,
            u64::from(actor.0) + 1,
            ClientCommand::Game(GuandanCommand::Act(action).into()),
        );
        assert!(
            !deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
        for delivery in deliveries {
            if let ServerEvent::GameSnapshot(snapshot) = delivery.message.event {
                let stats = snapshot
                    .player(actor)
                    .unwrap()
                    .game_profiles
                    .guandan
                    .as_ref()
                    .unwrap()
                    .action_time;
                assert_eq!(stats.completed_actions, before.completed_actions + 1);
                assert_eq!(stats.total_millis, before.total_millis + 61_000);
                assert_eq!(stats.longest_millis, 61_000);
                assert!(
                    delivery
                        .message
                        .action_timing
                        .unwrap()
                        .clocks
                        .iter()
                        .all(|c| c.elapsed_millis == 0)
                );
            }
        }
    }
}
