use crate::{
    Catalogue, EntryKind, GameState, GuessMode, GuessPool, MatchQuality, MinecraftGuessRuleSet,
};

#[test]
fn enchantment_applicability_compares_actual_item_memberships() {
    let applicable = |name| {
        Catalogue::bundled()
            .find(EntryKind::Enchantment, name)
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.label == "适用物品")
            .unwrap()
    };
    for (guess, answer, expected) in [
        ("迅捷潜行", "消失诅咒", MatchQuality::Partial),
        ("迅捷潜行", "绑定诅咒", MatchQuality::Partial),
        ("水下呼吸", "经验修补", MatchQuality::Partial),
        ("保护", "绑定诅咒", MatchQuality::Partial),
        ("经验修补", "耐久", MatchQuality::Exact),
        ("水下速掘", "水下呼吸", MatchQuality::Exact),
        ("迅捷潜行", "冰霜行者", MatchQuality::Wrong),
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
    let catalogue = Catalogue::bundled();
    assert!(
        catalogue
            .entries(GuessPool::Mixed)
            .all(|entry| { matches!(entry.kind, EntryKind::Block | EntryKind::Item) })
    );
    for pool in GuessPool::ALL {
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
    let answer = Catalogue::bundled()
        .find(EntryKind::VillagerProfession, "minecraft:farmer")
        .unwrap();
    for mode in [GuessMode::TurnBased, GuessMode::Race] {
        let mut rules = MinecraftGuessRuleSet {
            mode,
            ..Default::default()
        };
        rules.options_mut().pool = GuessPool::VillagerProfessions;
        let mut game = GameState::new(answer.clone(), vec![0, 1], rules);
        game.guess(0, "minecraft:librarian").unwrap();
        assert!(game.take_score_gains().is_empty());
        game.guess(1, "minecraft:armorer").unwrap();
        assert!(game.take_score_gains().is_empty());
        for action in game.take_action_statistics() {
            assert!(!action.all_wrong_after_first);
            assert!(!action.wrong_all_matching);
        }
        game.guess(0, "minecraft:farmer").unwrap();
        assert_eq!(game.view().winner, Some(0));
        assert!(!game.take_score_gains().is_empty());
    }
}
