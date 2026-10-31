use super::support::{accepted, bomb, game};
use crate::achievements::{DoudizhuAchievementFacts, DoudizhuPlayFacts};
use crate::{DoudizhuAction, DoudizhuCard, DoudizhuPhase, DoudizhuResultView, bot_action};
use leocard_game_common::PlayerId;

fn played(events: &[(PlayerId, DoudizhuAchievementFacts)]) -> DoudizhuPlayFacts {
    let DoudizhuAchievementFacts::Play(facts) = events[0].1 else {
        panic!("missing play report")
    };
    facts
}

#[test]
fn trick_bombs_reset_on_new_leads_but_consecutive_bombs_follow_personal_turns() {
    let (mut game, mut tracker) = game([
        [bomb(0), bomb(2), bomb(4), bomb(6)].concat(),
        [bomb(1), bomb(3), bomb(7)].concat(),
        vec![],
    ]);
    for (player, action) in [
        (0, DoudizhuAction::Play(bomb(0))),
        (1, DoudizhuAction::Play(bomb(1))),
        (2, DoudizhuAction::Pass),
        (0, DoudizhuAction::Play(bomb(2))),
        (1, DoudizhuAction::Play(bomb(3))),
        (2, DoudizhuAction::Pass),
    ] {
        accepted(&mut game, &mut tracker, PlayerId(player), action);
    }
    let facts = played(&accepted(
        &mut game,
        &mut tracker,
        PlayerId(0),
        DoudizhuAction::Play(bomb(4)),
    ));
    assert_eq!((facts.trick_bombs, facts.consecutive_bombs), (3, 3));
    accepted(&mut game, &mut tracker, PlayerId(1), DoudizhuAction::Pass);
    accepted(&mut game, &mut tracker, PlayerId(2), DoudizhuAction::Pass);
    let facts = played(&accepted(
        &mut game,
        &mut tracker,
        PlayerId(0),
        DoudizhuAction::Play(bomb(6)),
    ));
    assert!(facts.is_lead);
    assert_eq!((facts.trick_bombs, facts.consecutive_bombs), (1, 4));
    let facts = played(&accepted(
        &mut game,
        &mut tracker,
        PlayerId(1),
        DoudizhuAction::Play(bomb(7)),
    ));
    assert_eq!((facts.trick_bombs, facts.consecutive_bombs), (1, 1));
    accepted(&mut game, &mut tracker, PlayerId(2), DoudizhuAction::Pass);
    accepted(&mut game, &mut tracker, PlayerId(0), DoudizhuAction::Pass);
    let card = game.hand(PlayerId(1)).unwrap()[0];
    let facts = played(&accepted(
        &mut game,
        &mut tracker,
        PlayerId(1),
        DoudizhuAction::Play(vec![card]),
    ));
    assert_eq!(facts.consecutive_bombs, 0);
}

#[test]
fn flowers_require_the_current_farming_teammate_and_expire_after_that_turn() {
    let (mut game, mut tracker) = game([vec![], [bomb(1), bomb(3)].concat(), vec![]]);
    tracker.observe_flower(PlayerId(2), PlayerId(1), &game); // It is still the landlord's turn.
    let card = game.hand(PlayerId(0)).unwrap()[0];
    accepted(
        &mut game,
        &mut tracker,
        PlayerId(0),
        DoudizhuAction::Play(vec![card]),
    );
    tracker.observe_flower(PlayerId(0), PlayerId(1), &game);
    tracker.observe_flower(PlayerId(1), PlayerId(1), &game);
    tracker.observe_flower(PlayerId(2), PlayerId(1), &game);
    tracker.observe_flower(PlayerId(2), PlayerId(1), &game);
    let events = accepted(
        &mut game,
        &mut tracker,
        PlayerId(1),
        DoudizhuAction::Play(bomb(1)),
    );
    assert_eq!(
        events
            .iter()
            .filter(|(_, facts)| *facts == DoudizhuAchievementFacts::FlowerBomb)
            .copied()
            .collect::<Vec<_>>(),
        [(PlayerId(2), DoudizhuAchievementFacts::FlowerBomb)]
    );
    tracker.observe_flower(PlayerId(1), PlayerId(2), &game);
    accepted(&mut game, &mut tracker, PlayerId(2), DoudizhuAction::Pass);
    accepted(&mut game, &mut tracker, PlayerId(0), DoudizhuAction::Pass);
    let events = accepted(
        &mut game,
        &mut tracker,
        PlayerId(1),
        DoudizhuAction::Play(bomb(3)),
    );
    assert!(
        events
            .iter()
            .all(|(_, facts)| *facts != DoudizhuAchievementFacts::FlowerBomb)
    );
}

#[test]
fn opening_hand_is_only_reported_at_settlement_and_resets_between_hands() {
    let opening = [
        vec![
            DoudizhuCard(0),
            DoudizhuCard(4),
            DoudizhuCard(8),
            DoudizhuCard(12),
        ],
        (20..33).map(DoudizhuCard).collect(),
    ]
    .concat();
    let (mut game, mut tracker) = game([vec![], opening, vec![]]);
    for _ in 0..1000 {
        if game.phase == DoudizhuPhase::Finished {
            break;
        }
        let player = game.current;
        let action = bot_action(&game).unwrap();
        let events = accepted(&mut game, &mut tracker, player, action);
        assert!(
            events
                .iter()
                .all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Hand(_)))
        );
    }
    assert_eq!(game.phase, DoudizhuPhase::Finished);
    let after = game.result.as_ref().unwrap().scores.clone();
    let reports = tracker.finish_hand(&game, &[0; 3], &after, false);
    assert!(reports.iter().any(|(player, facts)| *player == PlayerId(1)
        && matches!(facts, DoudizhuAchievementFacts::Hand(facts) if facts.unavailable_exchange)));
    tracker.begin_hand();
    let reports = tracker.finish_hand(&game, &[0; 3], &after, false);
    assert!(reports.iter().all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Hand(facts) if facts.unavailable_exchange)));
}

#[test]
fn perfect_spring_requires_every_hand_and_rank_reversal_uses_only_final_scores() {
    let (mut game, mut tracker) = game([vec![], vec![], vec![]]);
    game.phase = DoudizhuPhase::Finished;
    game.result = Some(DoudizhuResultView {
        winner: PlayerId(0),
        landlord_won: true,
        spring: true,
        scores: vec![2, -1, -1],
    });
    let first = tracker.finish_hand(&game, &[0; 3], &[2, -1, -1], false);
    assert!(
        first
            .iter()
            .all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Match(_)))
    );
    tracker.begin_hand();
    game.result = Some(DoudizhuResultView {
        winner: PlayerId(1),
        landlord_won: false,
        spring: false,
        scores: vec![-8, 4, 4],
    });
    let last = tracker.finish_hand(&game, &[2, -1, -1], &[-6, 3, 3], true);
    assert!(last.iter().any(|(player, facts)| *player == PlayerId(0) && matches!(facts, DoudizhuAchievementFacts::Match(facts) if facts.first_to_last && !facts.perfect_spring)));
    assert!(last.iter().all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Match(facts) if facts.perfect_spring)));
    let (mut game, mut tracker) = super::support::game([vec![], vec![], vec![]]);
    game.phase = DoudizhuPhase::Finished;
    game.result = Some(DoudizhuResultView {
        winner: PlayerId(0),
        landlord_won: true,
        spring: true,
        scores: vec![2, -1, -1],
    });
    let reports = tracker.finish_hand(&game, &[0; 3], &[2, -1, -1], true);
    assert!(reports.iter().any(|(player, facts)| *player == PlayerId(0)
        && matches!(facts, DoudizhuAchievementFacts::Match(facts) if facts.perfect_spring)));
    assert!(reports.iter().any(|(player, facts)| *player == PlayerId(1)
        && matches!(facts, DoudizhuAchievementFacts::Hand(facts) if !facts.finished_cards)));
}
