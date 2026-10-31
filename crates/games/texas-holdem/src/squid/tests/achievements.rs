use crate::TexasSquidAwardStatistics;
use crate::achievements::{TexasAchievementEvent, register};
use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementRegistry, AchievementTrigger,
};
use leocard_game_common::PlayerId;

use std::sync::LazyLock;

static REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(|| {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    registry
});

fn award(book: &mut AchievementBook, sequence: u128, statistics: TexasSquidAwardStatistics) {
    book.trigger_with_registry(
        &AchievementTrigger::Game {
            player: PlayerId(0),
            event: TexasAchievementEvent::SquidAwarded {
                player: PlayerId(0),
                statistics,
            }
            .into(),
        },
        Some(AchievementContext {
            match_id: [1; 16],
            hand_index: Some(sequence as u32),
            sequence,
        }),
        sequence as u64,
        &REGISTRY,
    );
}

fn achieved(book: &AchievementBook, id: &str) -> bool {
    book.achieved(REGISTRY.by_id(id).unwrap())
}

#[test]
fn squid_achievement_thresholds_use_current_round_holdings_and_ignore_other_players() {
    let mut book = AchievementBook::default();
    let mut statistics = TexasSquidAwardStatistics {
        deepseek: false,
        owned: 2,
        deepseek_owned: 0,
    };
    // Two markers in each of two different rounds do not form a three-marker round.
    award(&mut book, 1, statistics);
    award(&mut book, 2, statistics);
    assert!(!achieved(&book, "leocard:texas/three_squid_in_round"));
    statistics.owned = 3;
    award(&mut book, 3, statistics);
    assert!(achieved(&book, "leocard:texas/three_squid_in_round"));
    assert!(!achieved(&book, "leocard:texas/five_squid_in_round"));
    assert!(!achieved(&book, "leocard:texas/first_deepseek_squid"));

    statistics.owned = 5;
    statistics.deepseek_owned = 4;
    statistics.deepseek = true;
    award(&mut book, 4, statistics);
    assert!(achieved(&book, "leocard:texas/five_squid_in_round"));
    assert!(achieved(&book, "leocard:texas/first_deepseek_squid"));
    assert!(!achieved(
        &book,
        "leocard:texas/five_deepseek_squid_in_round"
    ));
    statistics.deepseek_owned = 5;
    let event = TexasAchievementEvent::SquidAwarded {
        player: PlayerId(0),
        statistics,
    };
    let mut other = AchievementBook::default();
    let result = other.trigger_with_registry(
        &AchievementTrigger::Game {
            player: PlayerId(1),
            event: event.into(),
        },
        None,
        5,
        &REGISTRY,
    );
    assert!(result.unlocked.is_empty());
    award(&mut book, 5, statistics);
    assert!(achieved(
        &book,
        "leocard:texas/five_deepseek_squid_in_round"
    ));
}

#[test]
fn fifty_deepseek_squid_accumulate_across_rounds_with_live_event_deduplication() {
    let mut book = AchievementBook::default();
    let statistics = TexasSquidAwardStatistics {
        deepseek: true,
        owned: 1,
        deepseek_owned: 1,
    };
    let definition = REGISTRY
        .by_id("leocard:texas/fifty_deepseek_squid")
        .unwrap();
    for sequence in 1..=49 {
        award(&mut book, sequence, statistics);
        award(&mut book, sequence, statistics);
        assert_eq!(
            book.criterion_count(definition, "progress"),
            sequence as u64
        );
        assert!(!book.achieved(definition));
    }
    award(&mut book, 50, statistics);
    assert!(book.achieved(definition));
    assert!(!achieved(&book, "leocard:texas/three_squid_in_round"));
    assert!(!achieved(&book, "leocard:texas/five_squid_in_round"));
    assert!(!achieved(
        &book,
        "leocard:texas/five_deepseek_squid_in_round"
    ));
}
