use crate::achievements::{DEFINITIONS, GuessAchievementEvent, register};
use crate::{
    Catalogue, CorrectGuess, EntryKind, GameState, GuessActionStatistics, GuessPool,
    MinecraftGuessRuleSet,
};
use leocard_achievements::{
    AchievementBook, AchievementDefinition, AchievementRegistry, AchievementTrigger, ParticipantId,
};
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

fn event(statistics: GuessActionStatistics) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: ParticipantId(statistics.player),
        event: GuessAchievementEvent::ActionAnalyzed {
            player: ParticipantId(statistics.player),
            statistics: Box::new(statistics),
        }
        .into(),
    }
}

#[test]
fn first_green_reuses_the_existing_id_and_assists_require_another_player() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:blue_concrete")
        .unwrap()
        .clone();
    for same_player in [false, true] {
        let mut game = GameState::new(answer.clone(), vec![0, 1], MinecraftGuessRuleSet::default());
        let mut book = AchievementBook::default();
        game.guess(0, "minecraft:red_concrete").unwrap();
        let first = game.take_action_statistics().pop().unwrap();
        assert!(first.first_matching_wrong);
        book.trigger_with_registry(&event(first), None, 1, &REGISTRY);
        assert!(book.achieved(definition("matching_wrong")));
        assert_eq!(definition("matching_wrong").title, "MIT 协议");
        game.guess(1, "minecraft:green_concrete").unwrap();
        let second = game.take_action_statistics().pop().unwrap();
        assert!(!second.first_matching_wrong);
        if !same_player {
            // Player 0 also benefits from player 1's green guess.
            game.guess(0, "minecraft:blue_concrete").unwrap();
        } else {
            // With only player 0's green row, their own answer grants no assist.
            let mut fresh =
                GameState::new(answer.clone(), vec![0, 1], MinecraftGuessRuleSet::default());
            fresh.guess(0, "minecraft:red_concrete").unwrap();
            fresh.take_action_statistics();
            fresh.skip(1).unwrap();
            fresh.take_action_statistics();
            fresh.guess(0, "minecraft:blue_concrete").unwrap();
            game = fresh;
        }
        let correct = game.take_action_statistics().pop().unwrap();
        assert_eq!(correct.won_after_others_matching, !same_player);
        assert!(!correct.opening_win);
    }
}

#[test]
fn fifteen_wrong_greens_reward_all_participants_once_without_fabricating_guesses() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:blue_concrete")
        .unwrap()
        .clone();
    let mut game = GameState::new(answer, vec![0, 1, 2], MinecraftGuessRuleSet::default());
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
        if player == 2 {
            game.skip(2).unwrap();
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
        assert_eq!(recipients, if index == 14 { vec![0, 1] } else { vec![] });
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
        ("two_thousand_blocks", vec![EntryKind::Block], 2000),
        ("fifteen_hundred_items", vec![EntryKind::Item], 1500),
        (
            "five_hundred_world",
            vec![EntryKind::Landscape, EntryKind::Biome],
            500,
        ),
        (
            "five_hundred_magic",
            vec![EntryKind::Enchantment, EntryKind::Effect],
            500,
        ),
        (
            "two_hundred_professions",
            vec![EntryKind::VillagerProfession],
            200,
        ),
    ] {
        let mut book = AchievementBook::default();
        for index in 0..target {
            let kind = kinds[index % kinds.len()];
            let entry = Catalogue::bundled()
                .entries(
                    GuessPool::ALL
                        .into_iter()
                        .find(|pool| pool.contains(kind))
                        .unwrap(),
                )
                .next()
                .unwrap();
            book.trigger_with_registry(
                &event(GuessActionStatistics {
                    correct_entry: Some(CorrectGuess {
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
