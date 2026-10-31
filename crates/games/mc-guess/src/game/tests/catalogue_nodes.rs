use crate::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessMatchQuality, McGuessMode,
    McGuessPool, McGuessRuleSet,
};
use leocard_game_common::PlayerId;

#[test]
fn enchantment_applicability_compares_actual_item_memberships() {
    let applicable = |name| {
        McGuessCatalogue::bundled()
            .find(McGuessEntryKind::Enchantment, name)
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.label == "适用物品")
            .unwrap()
    };
    for (guess, answer, expected) in [
        ("迅捷潜行", "消失诅咒", McGuessMatchQuality::Partial),
        ("迅捷潜行", "绑定诅咒", McGuessMatchQuality::Partial),
        ("水下呼吸", "经验修补", McGuessMatchQuality::Partial),
        ("保护", "绑定诅咒", McGuessMatchQuality::Partial),
        ("经验修补", "耐久", McGuessMatchQuality::Exact),
        ("水下速掘", "水下呼吸", McGuessMatchQuality::Exact),
        ("迅捷潜行", "冰霜行者", McGuessMatchQuality::Wrong),
    ] {
        assert_eq!(applicable(guess).compare(applicable(answer)), expected);
        assert_eq!(applicable(answer).compare(applicable(guess)), expected);
    }
    assert_eq!(applicable("迅捷潜行").text(), "护腿");
    assert_eq!(
        applicable("消失诅咒").text(),
        "有耐久的物品、指南针、雕刻南瓜、生物头颅"
    );
}

#[test]
fn mixed_pool_excludes_the_development_catalogues() {
    let catalogue = McGuessCatalogue::bundled();
    assert!(
        catalogue.entries(McGuessPool::Mixed).all(|entry| {
            matches!(entry.kind, McGuessEntryKind::Block | McGuessEntryKind::Item)
        })
    );
    for pool in McGuessPool::ALL {
        assert!(catalogue.entries(pool).next().is_some());
        assert!(
            catalogue
                .entries(pool)
                .all(|entry| pool.contains(entry.kind))
        );
    }
}

#[test]
fn profession_guesses_have_no_attribute_or_matching_rewards() {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::VillagerProfession, "minecraft:farmer")
        .unwrap();
    for mode in [McGuessMode::TurnBased, McGuessMode::Race] {
        let mut rules = McGuessRuleSet {
            mode,
            ..Default::default()
        };
        rules.options_mut().pool = McGuessPool::VillagerProfessions;
        let mut game = McGuessGameState::new(answer.clone(), vec![PlayerId(0), PlayerId(1)], rules);
        game.guess(PlayerId(0), "minecraft:librarian").unwrap();
        assert!(game.take_score_gains().is_empty());
        game.guess(PlayerId(1), "minecraft:armorer").unwrap();
        assert!(game.take_score_gains().is_empty());
        for action in game.take_action_statistics() {
            assert!(!action.all_wrong_after_first);
            assert!(!action.wrong_all_matching);
        }
        game.guess(PlayerId(0), "minecraft:farmer").unwrap();
        assert_eq!(game.view().winner, Some(PlayerId(0)));
        assert!(!game.take_score_gains().is_empty());
    }
}
