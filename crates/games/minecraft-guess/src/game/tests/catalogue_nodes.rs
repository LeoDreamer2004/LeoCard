use crate::{Catalogue, EntryKind, GameState, GuessMode, GuessPool, MinecraftGuessRuleSet};

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
