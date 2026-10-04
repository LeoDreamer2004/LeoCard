use super::support::*;
use leocard_achievements::{
    ACHIEVEMENT_REGISTRY, AchievementContext, AchievementCriterion, AchievementScope,
    AchievementTier,
};
use leocard_protocol::MatchId;

#[test]
fn cumulative_thresholds_survive_reload_and_do_not_unlock_early() {
    let voice = chat(0, ChatContent::QuickVoice(0));
    for (trigger, silver, gold) in [
        (voice, "thousand_voices", "ten_thousand_voices"),
        (
            social(PlayerInteractionKind::Wine, 1, 0),
            "ten_thousand_flowers",
            "hundred_thousand_flowers",
        ),
        (
            social(PlayerInteractionKind::Shoe, 1, 0),
            "ten_thousand_eggs",
            "hundred_thousand_eggs",
        ),
    ] {
        let mut book = AchievementBook::default();
        for count in 1..=10_000 {
            book.trigger(&trigger, None, 100);
            if count == 500 {
                book = postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
            }
            assert_eq!(book.achieved(definition(silver)), count >= 1_000);
            assert_eq!(book.achieved(definition(gold)), count == 10_000);
        }
    }
    let mut book = AchievementBook::default();
    for count in 1..=10 {
        book.trigger(&social(PlayerInteractionKind::Wine, 0, 1), None, 100);
        assert_eq!(book.achieved(definition("flowers_sent")), count == 10);
    }
    book.trigger(&chat(0, ChatContent::Text("字".repeat(1000))), None, 100);
    assert!(!book.achieved(definition("text_chat")));
    book.trigger(&chat(0, ChatContent::Text("🙂".into())), None, 100);
    assert!(book.achieved(definition("text_chat")));
}

#[test]
fn only_host_start_facts_count_and_persistent_receipts_ignore_replays() {
    let mut book = AchievementBook::default();
    let started = AchievementTrigger::SessionStarted {
        player: PlayerId(0),
        host: PlayerId(0),
    };
    book.trigger(
        &AchievementTrigger::SessionStarted {
            player: PlayerId(0),
            host: PlayerId(1),
        },
        None,
        1,
    );
    assert!(!book.achieved(definition("first_host")));
    for count in 1..=500 {
        let context = Some(AchievementContext {
            match_id: MatchId([1; 16]),
            hand_index: None,
            sequence: count,
        });
        book.trigger(&started, context, 100);
        if count == 250 {
            book = postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
        }
        assert!(!book.trigger(&started, context, 101).progressed);
        assert_eq!(book.achieved(definition("five_hundred_host")), count == 500);
        assert_eq!(
            book.criterion_count(definition("five_hundred_host"), "progress"),
            count as u64
        );
    }
}

#[test]
fn fifth_gold_unlocks_the_meta_award_once_in_the_same_trigger() {
    const FIRST_FOUR: &[AchievementCriterion] = &[AchievementCriterion {
        id: "progress",
        amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("four"))),
        target: 1,
        scope: AchievementScope::Lifetime,
    }];
    const FIFTH: &[AchievementCriterion] = &[AchievementCriterion {
        id: "progress",
        amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("fifth"))),
        target: 1,
        scope: AchievementScope::Lifetime,
    }];
    let mut definitions = ACHIEVEMENT_REGISTRY
        .iter()
        .filter(|entry| {
            entry.tier == AchievementTier::Gold && entry.id != definition("five_gold").id
        })
        .take(5)
        .copied()
        .enumerate()
        .map(|(index, entry)| AchievementDefinition {
            criteria: if index < 4 { FIRST_FOUR } else { FIFTH },
            requirements: &[&["progress"]],
            ..entry
        })
        .collect::<Vec<_>>();
    definitions.push(*definition("five_gold"));
    let definitions: &'static [AchievementDefinition] = Box::leak(definitions.into_boxed_slice());
    let mut book = AchievementBook::default();
    book.trigger_with_registry(&AchievementTrigger::Signal("four"), None, 1, definitions);
    assert_eq!(book.counts().gold, 4);
    assert!(!book.achieved(definition("five_gold")));
    let result =
        book.trigger_with_registry(&AchievementTrigger::Signal("fifth"), None, 2, definitions);
    assert_eq!(result.unlocked.len(), 2);
    assert_eq!(book.counts().gold, 6);
    assert_eq!(book.earned_at(definition("five_gold")), Some(2));
    let mut book: AchievementBook =
        postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
    let counts = book.counts();
    assert!(
        !book
            .trigger(&AchievementTrigger::TrophyTotals(counts), None, 3)
            .progressed
    );
    assert_eq!(book.counts().gold, 6);
    let mut existing = AchievementBook::default();
    for signal in ["four", "fifth"] {
        existing.trigger_with_registry(
            &AchievementTrigger::Signal(signal),
            None,
            1,
            &definitions[..5],
        );
    }
    let mut existing: AchievementBook =
        postcard::from_bytes(&postcard::to_allocvec(&existing).unwrap()).unwrap();
    let totals = existing.counts();
    let result = existing.trigger(&AchievementTrigger::TrophyTotals(totals), None, 4);
    assert_eq!(result.unlocked.len(), 1);
    assert_eq!(result.unlocked[0].id, definition("five_gold").id);
}
