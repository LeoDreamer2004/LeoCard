use super::support::*;
use leocard_achievements::PersonalActivity;

fn observe(activity: &mut PersonalActivity, event: PlayerInteraction) -> bool {
    activity
        .observe_interaction(PlayerId(0), &event)
        .is_some_and(|event| amount("five_shoes", &AchievementTrigger::Personal(event)) == 1)
}

#[test]
fn shoe_window_uses_authoritative_times_and_includes_exactly_sixty_seconds() {
    for final_time in [60_000, 60_001] {
        let mut activity = PersonalActivity::default();
        for time in [0, 15_000, 30_000, 45_000] {
            assert!(!observe(
                &mut activity,
                interaction(PlayerInteractionKind::Shoe, 0, 1, time)
            ));
        }
        assert_eq!(
            observe(
                &mut activity,
                interaction(PlayerInteractionKind::Shoe, 0, 1, final_time)
            ),
            final_time == 60_000
        );
        // The next five consecutive actions may still form a newer valid window.
        assert!(observe(
            &mut activity,
            interaction(PlayerInteractionKind::Shoe, 0, 1, 70_000)
        ));
    }
}

#[test]
fn other_targets_and_outgoing_kinds_interrupt_but_incoming_actions_do_not() {
    for interruption in [
        interaction(PlayerInteractionKind::Shoe, 0, 2, 5),
        interaction(PlayerInteractionKind::Flower, 0, 1, 5),
        interaction(PlayerInteractionKind::Egg, 1, 0, 5),
    ] {
        let mut activity = PersonalActivity::default();
        for time in 0..4 {
            assert!(!observe(
                &mut activity,
                interaction(PlayerInteractionKind::Shoe, 0, 1, time)
            ));
        }
        assert!(!observe(&mut activity, interruption));
        assert_eq!(
            observe(
                &mut activity,
                interaction(PlayerInteractionKind::Shoe, 0, 1, 6)
            ),
            interruption.source != PlayerId(0)
        );
    }
    assert!(!observe(
        &mut PersonalActivity::default(),
        interaction(PlayerInteractionKind::Shoe, 0, 1, 0)
    ));
}
