use super::super::types::*;
use super::facts::*;
use leocard_mahjong::Fan;

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[AchievementDefinition {
    id: "leocard:mahjong/all_major_fans",
    category: AchievementCategory::Mahjong,
    title: "天选之子",
    tier: AchievementTier::Gold,
    description: "完成所有不低于64番的番种",
    criteria: &[
        AchievementCriterion {
            id: "BigFourWinds",
            amount: |event| u64::from(mahjong_fan(event, Fan::BigFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "BigThreeDragons",
            amount: |event| u64::from(mahjong_fan(event, Fan::BigThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllGreen",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllGreen)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "NineGates",
            amount: |event| u64::from(mahjong_fan(event, Fan::NineGates)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "FourKongs",
            amount: |event| u64::from(mahjong_fan(event, Fan::FourKongs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "SevenShiftedPairs",
            amount: |event| u64::from(mahjong_fan(event, Fan::SevenShiftedPairs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "ThirteenOrphans",
            amount: |event| u64::from(mahjong_fan(event, Fan::ThirteenOrphans)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllTerminals",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllTerminals)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "LittleFourWinds",
            amount: |event| u64::from(mahjong_fan(event, Fan::LittleFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "LittleThreeDragons",
            amount: |event| u64::from(mahjong_fan(event, Fan::LittleThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllHonors",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllHonors)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "FourConcealedPungs",
            amount: |event| u64::from(mahjong_fan(event, Fan::FourConcealedPungs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "PureTerminalChows",
            amount: |event| u64::from(mahjong_fan(event, Fan::PureTerminalChows)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
    ],
    requirements: &[
        &["BigFourWinds"],
        &["BigThreeDragons"],
        &["AllGreen"],
        &["NineGates"],
        &["FourKongs"],
        &["SevenShiftedPairs"],
        &["ThirteenOrphans"],
        &["AllTerminals"],
        &["LittleFourWinds"],
        &["LittleThreeDragons"],
        &["AllHonors"],
        &["FourConcealedPungs"],
        &["PureTerminalChows"],
    ],
}];
