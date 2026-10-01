use leocard_achievements::{
    AchievementBook, AchievementCategory, AchievementContext, AchievementCriterion,
    AchievementDefinition, AchievementScope, AchievementTier, AchievementTrigger,
};
use leocard_protocol::MatchId;
static COMBINED: &[AchievementDefinition] = &[AchievementDefinition {
    id: "test:combined",
    category: AchievementCategory::Mahjong,
    title: "组合条件",
    tier: AchievementTier::Gold,
    description: "任一启动条件及结束条件",
    criteria: &[
        AchievementCriterion {
            id: "start",
            amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("start"))),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "alternative",
            amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("alternative"))),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "finish",
            amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("finish"))),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
    ],
    requirements: &[&["start", "alternative"], &["finish"]],
}];

#[test]
fn mixed_and_or_criteria_survive_reload_and_support_non_fan_triggers() {
    let mut book = AchievementBook::default();
    let result = book.trigger_with_registry(
        &AchievementTrigger::Signal("alternative"),
        None,
        100,
        COMBINED,
    );
    assert!(result.progressed);
    assert!(result.unlocked.is_empty());
    let mut book: AchievementBook =
        postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
    assert_eq!(
        book.trigger_with_registry(&AchievementTrigger::Signal("finish"), None, 100, COMBINED)
            .unlocked
            .len(),
        1
    );
    assert!(
        book.trigger_with_registry(&AchievementTrigger::Signal("start"), None, 100, COMBINED)
            .unlocked
            .is_empty()
    );
}

fn scope_definitions(scope: AchievementScope) -> &'static [AchievementDefinition] {
    Box::leak(Box::new([AchievementDefinition {
        id: "test:scoped",
        category: AchievementCategory::Personal,
        title: "累计条件",
        tier: AchievementTier::Bronze,
        description: "两次累计及结束条件",
        criteria: Box::leak(Box::new([
            AchievementCriterion {
                id: "count",
                amount: |event| match event {
                    AchievementTrigger::Signal("tick") => 1,
                    AchievementTrigger::Signal("overflow") => u64::MAX,
                    _ => 0,
                },
                target: 2,
                scope,
            },
            AchievementCriterion {
                id: "finish",
                amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("finish"))),
                target: 1,
                scope,
            },
        ])),
        requirements: &[&["count"], &["finish"]],
    }]))
}

#[test]
fn scoped_counters_reset_and_sequenced_receipts_survive_reload() {
    for scope in [
        AchievementScope::Lifetime,
        AchievementScope::Match,
        AchievementScope::Hand,
    ] {
        let definitions = scope_definitions(scope);
        let mut book = AchievementBook::default();
        let context = |match_byte, hand, sequence| {
            Some(AchievementContext {
                match_id: MatchId([match_byte; 16]),
                hand_index: hand,
                sequence,
            })
        };
        let tick = AchievementTrigger::Signal("tick");
        book.trigger_with_registry(&tick, context(1, Some(0), 1), 10, definitions);
        let mut book: AchievementBook =
            postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
        assert!(
            !book
                .trigger_with_registry(&tick, context(1, Some(0), 1), 20, definitions)
                .progressed
        );
        assert_eq!(book.criterion_count(&definitions[0], "count"), 1);
        book.trigger_with_registry(&tick, context(1, Some(1), 2), 20, definitions);
        assert_eq!(
            book.criterion_count(&definitions[0], "count"),
            if scope == AchievementScope::Hand {
                1
            } else {
                2
            }
        );
        book.trigger_with_registry(&tick, context(2, Some(0), 1), 30, definitions);
        assert_eq!(
            book.criterion_count(&definitions[0], "count"),
            if scope == AchievementScope::Lifetime {
                2
            } else {
                1
            }
        );
        assert!(
            !book
                .trigger_with_registry(&tick, context(1, Some(0), 1), 40, definitions)
                .progressed
        );
        book.trigger_with_registry(
            &AchievementTrigger::Signal("overflow"),
            context(2, Some(0), 2),
            40,
            definitions,
        );
        assert_eq!(book.criterion_count(&definitions[0], "count"), 2);
        if scope != AchievementScope::Lifetime {
            assert!(
                book.trigger_with_registry(
                    &AchievementTrigger::Signal("finish"),
                    None,
                    50,
                    definitions
                )
                .unlocked
                .is_empty()
            );
        }
        assert_eq!(
            book.trigger_with_registry(
                &AchievementTrigger::Signal("finish"),
                context(2, Some(0), 3),
                60,
                definitions
            )
            .unlocked
            .len(),
            1
        );
        assert_eq!(book.earned_at(&definitions[0]), Some(60));
        book.trigger_with_registry(&tick, context(3, Some(0), 1), 70, definitions);
        assert_eq!(book.earned_at(&definitions[0]), Some(60));
    }
}

#[test]
fn missing_hand_context_cannot_combine_stale_hand_and_current_match_criteria() {
    let definitions = Box::leak(Box::new([AchievementDefinition {
        id: "test:mixed_scope",
        category: AchievementCategory::Personal,
        title: "不同范围",
        tier: AchievementTier::Bronze,
        description: "组合",
        criteria: &[
            AchievementCriterion {
                id: "hand",
                amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("hand"))),
                target: 1,
                scope: AchievementScope::Hand,
            },
            AchievementCriterion {
                id: "match",
                amount: |event| u64::from(matches!(event, AchievementTrigger::Signal("match"))),
                target: 1,
                scope: AchievementScope::Match,
            },
        ],
        requirements: &[&["hand"], &["match"]],
    }]));
    let mut book = AchievementBook::default();
    book.trigger_with_registry(
        &AchievementTrigger::Signal("hand"),
        Some(AchievementContext {
            match_id: MatchId([1; 16]),
            hand_index: Some(0),
            sequence: 1,
        }),
        10,
        definitions,
    );
    assert!(
        book.trigger_with_registry(
            &AchievementTrigger::Signal("match"),
            Some(AchievementContext {
                match_id: MatchId([2; 16]),
                hand_index: None,
                sequence: 1
            }),
            20,
            definitions
        )
        .unlocked
        .is_empty()
    );
}
