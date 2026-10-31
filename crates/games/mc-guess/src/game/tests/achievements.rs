use crate::achievements::{DEFINITIONS, McGuessAchievementEvent, register};
use crate::{
    McGuessActionStatistics, McGuessCatalogue, McGuessCorrectGuess, McGuessEntryKind,
    McGuessGameState, McGuessPool, McGuessRuleSet,
};
use leocard_achievements::{
    AchievementBook, AchievementDefinition, AchievementRegistry, AchievementTrigger,
};
use leocard_game_common::PlayerId;

use std::sync::LazyLock;

static REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(|| {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    registry
});

fn definition(id: &str) -> &'static AchievementDefinition {
    DEFINITIONS
        .iter()
        .find(|definition| definition.id.ends_with(&format!("/{id}")))
        .unwrap()
}

fn event(statistics: McGuessActionStatistics) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: statistics.player,
        event: McGuessAchievementEvent::ActionAnalyzed {
            player: statistics.player,
            statistics: Box::new(statistics),
        }
        .into(),
    }
}

#[test]
fn first_green_reuses_the_existing_id_and_assists_require_another_player() {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:blue_concrete")
        .unwrap()
        .clone();
    for same_player in [false, true] {
        let mut game = McGuessGameState::new(
            answer.clone(),
            vec![PlayerId(0), PlayerId(1)],
            McGuessRuleSet::default(),
        );
        let mut book = AchievementBook::default();
        game.guess(PlayerId(0), "minecraft:red_concrete").unwrap();
        let first = game.take_action_statistics().pop().unwrap();
        assert!(first.first_matching_wrong);
        book.trigger_with_registry(&event(first), None, 1, &REGISTRY);
        assert!(book.achieved(definition("matching_wrong")));
        assert_eq!(definition("matching_wrong").title, "MIT 协议");
        game.guess(PlayerId(1), "minecraft:green_concrete").unwrap();
        let second = game.take_action_statistics().pop().unwrap();
        assert!(!second.first_matching_wrong);
        if !same_player {
            // Player 0 also benefits from player 1's green guess.
            game.guess(PlayerId(0), "minecraft:blue_concrete").unwrap();
        } else {
            // With only player 0's green row, their own answer grants no assist.
            let mut fresh = McGuessGameState::new(
                answer.clone(),
                vec![PlayerId(0), PlayerId(1)],
                McGuessRuleSet::default(),
            );
            fresh.guess(PlayerId(0), "minecraft:red_concrete").unwrap();
            fresh.take_action_statistics();
            fresh.skip(PlayerId(1)).unwrap();
            fresh.take_action_statistics();
            fresh.guess(PlayerId(0), "minecraft:blue_concrete").unwrap();
            game = fresh;
        }
        let correct = game.take_action_statistics().pop().unwrap();
        assert_eq!(correct.won_after_others_matching, !same_player);
        assert!(!correct.opening_win);
    }
}

#[test]
fn fifteen_wrong_greens_reward_all_participants_once_without_fabricating_guesses() {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:blue_concrete")
        .unwrap()
        .clone();
    let mut game = McGuessGameState::new(
        answer,
        vec![PlayerId(0), PlayerId(1), PlayerId(2)],
        McGuessRuleSet::default(),
    );
    let colors = [
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "brown",
        "green",
        "red",
        "black",
    ];
    for (index, color) in colors.into_iter().enumerate() {
        // Player 2 never contributes a green row.
        let player = game.current_player().unwrap();
        if player == PlayerId(2) {
            game.skip(PlayerId(2)).unwrap();
            game.take_action_statistics();
        }
        let player = game.current_player().unwrap();
        game.guess(player, &format!("minecraft:{color}_concrete"))
            .unwrap();
        let actions = game.take_action_statistics();
        let recipients: Vec<_> = actions
            .iter()
            .filter(|action| action.unlucky_matching)
            .map(|action| action.player)
            .collect();
        assert_eq!(
            recipients,
            if index == 14 {
                vec![PlayerId(0), PlayerId(1)]
            } else {
                vec![]
            }
        );
    }
    assert!(!game.finished());
    game.guess(game.current_player().unwrap(), "minecraft:blue_concrete")
        .unwrap();
    assert!(
        game.take_action_statistics()
            .iter()
            .all(|action| !action.unlucky_matching)
    );
}

#[test]
fn category_counters_accumulate_both_kinds_and_retain_progress_between_events() {
    for (id, kinds, target) in [
        ("two_thousand_blocks", vec![McGuessEntryKind::Block], 2000),
        ("fifteen_hundred_items", vec![McGuessEntryKind::Item], 1500),
        (
            "five_hundred_world",
            vec![McGuessEntryKind::Landscape, McGuessEntryKind::Biome],
            500,
        ),
        (
            "five_hundred_magic",
            vec![McGuessEntryKind::Enchantment, McGuessEntryKind::Effect],
            500,
        ),
        (
            "two_hundred_professions",
            vec![McGuessEntryKind::VillagerProfession],
            200,
        ),
    ] {
        let mut book = AchievementBook::default();
        for index in 0..target {
            let kind = kinds[index % kinds.len()];
            let entry = McGuessCatalogue::bundled()
                .entries(
                    McGuessPool::ALL
                        .into_iter()
                        .find(|pool| pool.contains(kind))
                        .unwrap(),
                )
                .next()
                .unwrap();
            book.trigger_with_registry(
                &event(McGuessActionStatistics {
                    correct_entry: Some(McGuessCorrectGuess {
                        id: entry.id.clone(),
                        kind,
                    }),
                    ..Default::default()
                }),
                None,
                1,
                &REGISTRY,
            );
            assert_eq!(book.achieved(definition(id)), index + 1 == target);
        }
    }
}
