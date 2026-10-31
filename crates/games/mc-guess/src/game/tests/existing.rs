use super::super::{McGuessGameError, McGuessGameState, McGuessResult};
use crate::{
    McGuessAttribute, McGuessCatalogue, McGuessEntryKind, McGuessMatchQuality, McGuessPool,
    McGuessRuleSet, McGuessValueDirection,
};
use leocard_game_common::PlayerId;

fn attribute(values: &[&str]) -> McGuessAttribute {
    McGuessAttribute {
        label: "分类".to_owned(),
        values: values.iter().map(|value| (*value).to_owned()).collect(),
    }
}

#[test]
fn multiple_values_compare_as_sets() {
    let target = attribute(&["自然生成", "合成"]);
    assert_eq!(
        attribute(&["合成", "自然生成"]).compare(&target),
        McGuessMatchQuality::Exact
    );
    assert_eq!(
        attribute(&["合成", "战利品"]).compare(&target),
        McGuessMatchQuality::Partial
    );
    assert_eq!(
        attribute(&["战利品"]).compare(&target),
        McGuessMatchQuality::Wrong
    );
}

#[test]
fn ordered_feedback_points_toward_the_answer_without_revealing_it() {
    for (label, guess, answer, direction) in [
        ("硬度", "1.5", "2", Some(McGuessValueDirection::Up)),
        ("硬度", "-1", "50", Some(McGuessValueDirection::Down)),
        ("爆炸抗性", "1200", "6", Some(McGuessValueDirection::Down)),
        ("堆叠数量", "1", "64", Some(McGuessValueDirection::Up)),
        (
            "首次加入版本",
            "1.9",
            "1.10",
            Some(McGuessValueDirection::Up),
        ),
        (
            "首次加入版本",
            "26.1",
            "1.21",
            Some(McGuessValueDirection::Down),
        ),
        (
            "首次加入版本",
            "pre-classic",
            "1.1",
            Some(McGuessValueDirection::Up),
        ),
        ("稀有度", "普通", "稀有", Some(McGuessValueDirection::Up)),
        ("硬度", "2", "2", None),
        ("挖掘工具", "镐", "斧", None),
    ] {
        let mut ours = attribute(&[guess]);
        let mut theirs = attribute(&[answer]);
        ours.label = label.to_owned();
        theirs.label = label.to_owned();
        assert_eq!(
            ours.feedback(&theirs).direction,
            direction,
            "{label}: {guess}"
        );
    }
}

#[test]
fn catalogue_keeps_playable_entries_and_excludes_technical_variants() {
    let catalogue = McGuessCatalogue::bundled();
    for id in [
        "command_block",
        "repeating_command_block",
        "chain_command_block",
        "structure_block",
        "jigsaw",
        "torch",
        "oak_sign",
        "cobblestone_wall",
        "candle",
        "cake",
        "cauldron",
        "kelp",
        "potted_dandelion",
    ] {
        assert!(
            catalogue
                .find(McGuessEntryKind::Block, &format!("minecraft:{id}"))
                .is_some()
        );
    }
    for id in [
        "air",
        "cave_air",
        "void_air",
        "moving_piston",
        "piston_head",
        "test_block",
        "test_instance_block",
        "wall_torch",
        "oak_wall_sign",
        "oak_wall_hanging_sign",
        "skeleton_wall_skull",
        "white_wall_banner",
        "tube_coral_wall_fan",
        "candle_cake",
        "red_candle_cake",
        "water_cauldron",
        "attached_pumpkin_stem",
        "kelp_plant",
    ] {
        assert!(
            catalogue
                .find(McGuessEntryKind::Block, &format!("minecraft:{id}"))
                .is_none()
        );
    }
    for entry in catalogue.entries(McGuessPool::Mixed) {
        assert!(
            !entry
                .attributes
                .iter()
                .any(|attribute| attribute.label == "主色调")
        );
        assert!(!entry.attributes.is_empty());
        assert!(
            entry
                .attributes
                .iter()
                .all(|attribute| !attribute.values.is_empty())
        );
    }
    assert!(
        catalogue
            .find(McGuessEntryKind::Block, "minecraft:wheat")
            .is_none()
    );
    assert!(
        catalogue
            .find(McGuessEntryKind::Item, "minecraft:wheat")
            .is_some()
    );
}

#[test]
fn turns_keep_answer_private_until_a_correct_guess_and_reject_invalid_actions() {
    let catalogue = McGuessCatalogue::bundled();
    let answer = catalogue
        .find(McGuessEntryKind::Block, "minecraft:oak_log")
        .unwrap()
        .clone();
    let mut game = McGuessGameState::new(
        answer.clone(),
        vec![PlayerId(2), PlayerId(5)],
        McGuessRuleSet::default(),
    );
    assert!(game.view().answer.is_none());
    assert_eq!(
        game.view().character_count,
        answer.name.chars().count() as u16
    );
    assert_eq!(
        game.guess(PlayerId(5), "minecraft:stone"),
        Err(McGuessGameError::NotPlayersTurn)
    );
    assert_eq!(
        game.guess(PlayerId(2), "不存在"),
        Err(McGuessGameError::UnknownEntry)
    );
    assert_eq!(game.current_player(), Some(PlayerId(2)));
    game.guess(PlayerId(2), "minecraft:stone").unwrap();
    assert_eq!(game.current_player(), Some(PlayerId(5)));
    assert!(game.view().answer.is_none());
    assert_eq!(
        game.guess(PlayerId(5), "minecraft:stone"),
        Err(McGuessGameError::AlreadyGuessed)
    );
    game.guess(PlayerId(5), &answer.name).unwrap();
    let view = game.view();
    assert_eq!(view.winner, Some(PlayerId(5)));
    assert_eq!(view.answer, Some(answer));
    assert!(view.current_player.is_none());
    let McGuessResult::Known { matches, .. } = &view.guesses[1].result else {
        panic!("correct guess has known attributes");
    };
    assert!(
        matches
            .iter()
            .all(|feedback| feedback.quality == McGuessMatchQuality::Exact
                && feedback.direction.is_none())
    );
    assert_eq!(game.skip(PlayerId(5)), Err(McGuessGameError::Finished));
}

#[test]
fn skipping_advances_and_a_full_pass_reveals_the_answer() {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Item, "minecraft:stick")
        .unwrap()
        .clone();
    let mut game = McGuessGameState::new(
        answer.clone(),
        vec![PlayerId(0), PlayerId(1)],
        McGuessRuleSet::default(),
    );
    game.skip(PlayerId(0)).unwrap();
    assert_eq!(game.current_player(), Some(PlayerId(1)));
    game.guess(PlayerId(1), "minecraft:diamond").unwrap();
    assert!(game.view().skipped_players.is_empty());
    game.skip(PlayerId(0)).unwrap();
    game.skip(PlayerId(1)).unwrap();
    assert_eq!(game.view().answer, Some(answer));
    assert_eq!(game.view().winner, None);
    assert_eq!(game.view().guesses.len(), 1);
}
