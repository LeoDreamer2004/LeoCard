use super::{facts::*, macros::shengji};
use crate::registry::{macros::award, types::*};
use leocard_shengji::{Component, ShengjiRank};

pub(super) const DEFINITIONS: &[AchievementDefinition] =
    &[
        shengji!(
            "all_points",
            "应有尽有",
            Silver,
            "坐闲在底上没有埋分的情况下拿下所有分牌",
            |event| hand(event, |facts| attacking(facts)
                && facts.result.kitty_points == 0
                && facts.result.trick_points == 100 * u32::from(facts.deck_count)),
            1
        ),
        AchievementDefinition {
            id: "leocard:shengji/all_decks",
            category: AchievementCategory::Shengji,
            title: "升级全才",
            tier: AchievementTier::Silver,
            description: "2/3/4副各完成20盘",
            criteria: &[
                AchievementCriterion {
                    id: "two",
                    amount: |event| completed(event, 2),
                    target: 20,
                    scope: AchievementScope::Lifetime,
                },
                AchievementCriterion {
                    id: "three",
                    amount: |event| completed(event, 3),
                    target: 20,
                    scope: AchievementScope::Lifetime,
                },
                AchievementCriterion {
                    id: "four",
                    amount: |event| completed(event, 4),
                    target: 20,
                    scope: AchievementScope::Lifetime,
                },
            ],
            requirements: &[&["two"], &["three"], &["four"]],
        },
        shengji!(
            "ace",
            "激情酣战",
            Silver,
            "一场游戏自己方至少打到A",
            |event| hand(event, |facts| facts.result.levels
                [usize::from(facts.player.team().0)]
                == ShengjiRank::Ace),
            1
        ),
        shengji!(
            "spaceship",
            "没见过的孩子呢",
            Silver,
            "在4副牌中打出宇宙飞船",
            |event| play(event, |facts, _| facts
                .play
                .components
                .iter()
                .any(|component| matches!(component, Component::Spaceship { .. }))),
            1
        ),
        shengji!(
            "throw_twelve",
            "琳琅满目",
            Silver,
            "甩牌超过12张",
            |event| thrown_over(event, 12),
            1
        ),
        shengji!(
            "scoring_bottom_defended",
            "艺高人胆大",
            Silver,
            "坐庄在底牌中所有牌都是分牌的情况下保底",
            |event| hand(event, |facts| facts.player == facts.result.dealer
                && facts.bottom_burier == Some(facts.player)
                && facts.result.kitty_multiplier == 0
                && facts.burial.is_some_and(|burial| burial.card_count > 0
                    && burial.card_count == burial.scoring_card_count)),
            1
        ),
        shengji!(
            "attack_thirteen",
            "惊不惊喜，意不意外",
            Silver,
            "坐闲一局升级不少于13级",
            |event| hand(event, |facts| attack_promotes(facts, 13)),
            1
        ),
    ];
