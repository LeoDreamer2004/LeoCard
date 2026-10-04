use super::support::*;
use leocard_achievements::{AchievementCategory, achievements_in};
use leocard_protocol::MatchId;
use leocard_shengji::ShengjiRedealReason;

#[test]
fn deck_mastery_counts_each_deck_independently_and_ignores_replays() {
    assert_eq!(achievements_in(AchievementCategory::Shengji).count(), 28);
    let mut book = AchievementBook::default();
    let mastery = achievement_by_id("leocard:shengji/all_decks").unwrap();
    let mut sequence = 0;
    for deck in 2..=4 {
        for hand in 1..=20 {
            sequence += 1;
            let mut facts = statistics(1);
            facts.deck_count = deck;
            let context = Some(AchievementContext {
                match_id: MatchId([1; 16]),
                hand_index: Some(sequence),
                sequence: u128::from(sequence),
            });
            let trigger = analyzed(facts);
            book.trigger(&trigger, context, 100);
            assert!(!book.trigger(&trigger, context, 100).progressed);
            assert_eq!(book.achieved(mastery), deck == 4 && hand == 20);
            let small = achievement_by_id(&format!(
                "leocard:shengji/{}_deck_ten",
                ["two", "three", "four"][usize::from(deck - 2)]
            ))
            .unwrap();
            assert_eq!(book.achieved(small), hand >= 10);
        }
    }
    for id in ["two", "three", "four"] {
        assert_eq!(book.criterion_count(mastery, id), 20);
    }
}

#[test]
fn lifetime_total_uses_settled_hands_and_survives_reload() {
    let definition = achievement_by_id("leocard:shengji/ten_thousand").unwrap();
    let mut book = AchievementBook::default();
    let trigger = analyzed(statistics(1));
    for count in 1..=10_000 {
        book.trigger(
            &trigger,
            Some(AchievementContext {
                match_id: MatchId([2; 16]),
                hand_index: Some(count),
                sequence: u128::from(count),
            }),
            100,
        );
        if count == 5_000 {
            book = postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
        }
        assert_eq!(book.achieved(definition), count == 10_000);
    }
    assert_eq!(book.criterion_count(definition, "progress"), 10_000);
    let redeal = AchievementTrigger::Game {
        player: PlayerId(1),
        event: ShengjiEvent::RedealRequired {
            reason: ShengjiRedealReason::NoDeclaration,
        }
        .into(),
    };
    assert_eq!(amount("ten_thousand", &redeal), 0);
}
