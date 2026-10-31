use crate::achievements::{
    DoudizhuAchievementEvent, DoudizhuAchievementFacts, DoudizhuHandFacts, DoudizhuMatchFacts,
    DoudizhuPlayFacts, register,
};
use crate::{DoudizhuMode, DoudizhuPlayKind};
use leocard_achievements::{
    AchievementBook, AchievementCategory, AchievementContext, AchievementRegistry,
    AchievementTrigger,
};
use leocard_game_common::PlayerId;

use std::collections::HashSet;
use std::sync::LazyLock;

static REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(|| {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    registry
});

fn play(kind: DoudizhuPlayKind, card_count: u8) -> DoudizhuAchievementFacts {
    DoudizhuAchievementFacts::Play(DoudizhuPlayFacts {
        mode: DoudizhuMode::Three,
        kind,
        card_count,
        rank: 11,
        is_lead: true,
        beats_airplane: true,
        trick_bombs: 3,
        consecutive_bombs: 3,
    })
}

fn trigger(player: PlayerId, facts: DoudizhuAchievementFacts) -> AchievementTrigger {
    AchievementTrigger::Game {
        player,
        event: DoudizhuAchievementEvent::ActionAnalyzed {
            player: PlayerId(0),
            statistics: Box::new(facts),
        }
        .into(),
    }
}

#[test]
fn all_documented_achievements_register_and_unlock_for_the_right_recipient() {
    let hand = DoudizhuHandFacts {
        landlord: true,
        won: true,
        spring: true,
        finished_cards: true,
        finishing_bomb: true,
        unavailable_exchange: true,
        all_bomb_hand: true,
        play_count: 2,
        score: 50,
    };
    let whole = DoudizhuAchievementFacts::Match(DoudizhuMatchFacts {
        score: 500,
        perfect_spring: true,
        first_to_last: true,
    });
    let mut six = play(DoudizhuPlayKind::Single, 1);
    if let DoudizhuAchievementFacts::Play(facts) = &mut six {
        facts.rank = 3;
    }
    let cases = [
        ("landlord_win", DoudizhuAchievementFacts::Hand(hand)),
        (
            "farmer_win",
            DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                landlord: false,
                ..hand
            }),
        ),
        ("three_player_bomb", play(DoudizhuPlayKind::Bomb, 4)),
        ("airplane", play(DoudizhuPlayKind::AirplaneSingles, 8)),
        ("higher_airplane", play(DoudizhuPlayKind::AirplanePairs, 10)),
        ("landlord_spring", DoudizhuAchievementFacts::Hand(hand)),
        (
            "farmer_spring",
            DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                landlord: false,
                ..hand
            }),
        ),
        ("lead_six", six),
        ("bomb_finish", DoudizhuAchievementFacts::Hand(hand)),
        ("unavailable_exchange", DoudizhuAchievementFacts::Hand(hand)),
        ("flower_bomb", DoudizhuAchievementFacts::FlowerBomb),
        ("first_to_last", whole),
        ("three_bombs_in_trick", play(DoudizhuPlayKind::Bomb, 4)),
        ("full_straight", play(DoudizhuPlayKind::Straight, 12)),
        ("seven_pairs", play(DoudizhuPlayKind::Pairs, 14)),
        ("four_airplane", play(DoudizhuPlayKind::Airplane, 12)),
        ("fifty_points", DoudizhuAchievementFacts::Hand(hand)),
        ("three_consecutive_bombs", play(DoudizhuPlayKind::Bomb, 4)),
        ("perfect_spring", whole),
        ("all_bomb_hand", DoudizhuAchievementFacts::Hand(hand)),
        ("five_hundred_points", whole),
        ("two_plays", DoudizhuAchievementFacts::Hand(hand)),
    ];
    let ids = cases
        .iter()
        .map(|(id, _)| *id)
        .chain(["five_thousand_farmer_wins", "five_thousand_landlord_wins"])
        .map(|id| format!("leocard:doudizhu/{id}"))
        .collect::<HashSet<_>>();
    assert_eq!(
        ids,
        REGISTRY
            .in_category(AchievementCategory::Doudizhu)
            .map(|definition| definition.id.to_owned())
            .collect()
    );
    for (id, facts) in cases {
        let definition = REGISTRY.by_id(&format!("leocard:doudizhu/{id}")).unwrap();
        let mut book = AchievementBook::default();
        assert!(
            book.trigger_with_registry(&trigger(PlayerId(1), facts), None, 1, &REGISTRY)
                .unlocked
                .is_empty()
        );
        let event = trigger(PlayerId(0), facts);
        let context = Some(AchievementContext {
            match_id: [1; 16],
            hand_index: Some(1),
            sequence: 1,
        });
        book.trigger_with_registry(&event, context, 1, &REGISTRY);
        assert!(book.achieved(definition), "{id}");
        assert!(
            book.trigger_with_registry(&event, context, 2, &REGISTRY)
                .unlocked
                .is_empty()
        );
    }
}

#[test]
fn thresholds_distinguish_bomb_modes_wings_team_wins_and_net_scores() {
    let mut four_bomb = play(DoudizhuPlayKind::Bomb, 4);
    if let DoudizhuAchievementFacts::Play(facts) = &mut four_bomb {
        facts.mode = DoudizhuMode::Four;
    }
    let mut following_six = play(DoudizhuPlayKind::Single, 1);
    if let DoudizhuAchievementFacts::Play(facts) = &mut following_six {
        facts.rank = 3;
        facts.is_lead = false;
    }
    let cases = [
        ("three_player_bomb", four_bomb),
        ("lead_six", following_six),
        ("four_airplane", play(DoudizhuPlayKind::AirplanePairs, 15)),
        ("seven_pairs", play(DoudizhuPlayKind::Pairs, 12)),
        ("full_straight", play(DoudizhuPlayKind::Straight, 11)),
        (
            "fifty_points",
            DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                score: 49,
                ..Default::default()
            }),
        ),
        (
            "five_hundred_points",
            DoudizhuAchievementFacts::Match(DoudizhuMatchFacts {
                score: 499,
                ..Default::default()
            }),
        ),
        (
            "two_plays",
            DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                won: true,
                play_count: 1,
                ..Default::default()
            }),
        ),
        (
            "two_plays",
            DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                won: true,
                finished_cards: true,
                play_count: 3,
                ..Default::default()
            }),
        ),
    ];
    for (id, facts) in cases {
        let definition = REGISTRY.by_id(&format!("leocard:doudizhu/{id}")).unwrap();
        let mut book = AchievementBook::default();
        book.trigger_with_registry(&trigger(PlayerId(0), facts), None, 1, &REGISTRY);
        assert!(!book.achieved(definition), "{id}");
    }
}

#[test]
fn landlord_and_farmer_wins_accumulate_separately_across_matches_and_deduplicate_live_facts() {
    for (landlord, id) in [
        (false, "five_thousand_farmer_wins"),
        (true, "five_thousand_landlord_wins"),
    ] {
        let definition = REGISTRY.by_id(&format!("leocard:doudizhu/{id}")).unwrap();
        let mut book = AchievementBook::default();
        let facts = DoudizhuHandFacts {
            landlord,
            won: true,
            ..Default::default()
        };
        let event = trigger(PlayerId(0), DoudizhuAchievementFacts::Hand(facts));
        for sequence in 1..5000 {
            let context = Some(AchievementContext {
                match_id: [u8::from(sequence > 2500); 16],
                hand_index: Some(sequence as u32),
                sequence,
            });
            book.trigger_with_registry(&event, context, sequence as u64, &REGISTRY);
        }
        assert_eq!(book.criterion_count(definition, "progress"), 4999);
        assert!(!book.achieved(definition));
        for invalid in [
            DoudizhuHandFacts {
                won: false,
                ..facts
            },
            DoudizhuHandFacts {
                landlord: !landlord,
                ..facts
            },
        ] {
            book.trigger_with_registry(
                &trigger(PlayerId(0), DoudizhuAchievementFacts::Hand(invalid)),
                None,
                5000,
                &REGISTRY,
            );
        }
        assert_eq!(book.criterion_count(definition, "progress"), 4999);
        let context = Some(AchievementContext {
            match_id: [1; 16],
            hand_index: Some(5000),
            sequence: 5000,
        });
        let update = book.trigger_with_registry(&event, context, 5000, &REGISTRY);
        assert!(update.unlocked.iter().any(|item| item.id == definition.id));
        assert!(book.achieved(definition));
        assert_eq!(book.criterion_count(definition, "progress"), 5000);
        assert!(
            book.trigger_with_registry(&event, context, 5001, &REGISTRY)
                .unlocked
                .is_empty()
        );
    }
    assert_eq!(
        REGISTRY
            .by_id("leocard:doudizhu/first_to_last")
            .unwrap()
            .title,
        "玩游戏要啸着玩..."
    );
}
