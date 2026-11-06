use super::*;
use leocard_protocol::{TexasCommand, TexasPhaseView};
use leocard_texas_holdem::{TexasAction, TexasRuleSet};

#[test]
fn all_in_players_get_separate_human_decision_times_for_runout_votes() {
    let mut table = Table::new_for(GameRules::Texas(TexasRuleSet {
        side_pots: false,
        run_it_twice: true,
        ..Default::default()
    }));
    for _ in 0..40 {
        if !table.view().1.clocks.is_empty() {
            break;
        }
        table.session.advance_time(Duration::from_millis(100));
    }
    for _ in 0..4 {
        let GameSnapshot::Texas(snapshot) = table.view().0 else {
            unreachable!()
        };
        if matches!(snapshot.phase, TexasPhaseView::ChoosingRunout) {
            break;
        }
        let actor = snapshot.current_player.unwrap();
        let deliveries = table.send(
            u64::from(actor.0) + 1,
            ClientCommand::Game(
                TexasCommand::Act {
                    action: TexasAction::AllIn,
                }
                .into(),
            ),
        );
        assert!(
            !deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
    }
    for _ in 0..4 {
        let (GameSnapshot::Texas(snapshot), timing) = table.view() else {
            unreachable!()
        };
        assert!(matches!(snapshot.phase, TexasPhaseView::ChoosingRunout));
        let actor = snapshot.current_player.unwrap();
        assert!(
            snapshot
                .players
                .iter()
                .find(|p| p.player.id == actor)
                .unwrap()
                .all_in
        );
        assert_eq!(timing.clocks.len(), 1);
        assert_eq!(timing.clocks[0].player, actor);
        assert_eq!(timing.clocks[0].elapsed_millis, 0);
        let before = snapshot
            .players
            .iter()
            .find(|p| p.player.id == actor)
            .unwrap()
            .player
            .game_profiles
            .action_time(GameKind::Texas);
        table.session.advance_time(Duration::from_millis(750));
        let deliveries = table.send(
            u64::from(actor.0) + 1,
            ClientCommand::Game(TexasCommand::ChooseRunout { twice: true }.into()),
        );
        assert!(
            !deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
        let (snapshot, _) = table.view();
        let after = snapshot
            .player(actor)
            .unwrap()
            .game_profiles
            .action_time(GameKind::Texas);
        assert_eq!(after.completed_actions, before.completed_actions + 1);
        assert_eq!(after.total_millis, before.total_millis + 750);
    }
    let (GameSnapshot::Texas(snapshot), timing) = table.view() else {
        unreachable!()
    };
    assert!(matches!(
        snapshot.phase,
        TexasPhaseView::HandComplete { .. }
    ));
    assert!(timing.clocks.is_empty());
}
