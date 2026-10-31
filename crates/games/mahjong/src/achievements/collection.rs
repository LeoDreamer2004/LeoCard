use super::facts::*;
use crate::MahjongFan;
use leocard_achievements::{
    AchievementCategory, AchievementCriterion, AchievementDefinition, AchievementScope,
    AchievementTier,
};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[AchievementDefinition {
    id: "leocard:mahjong/all_major_fans",
    category: AchievementCategory::Mahjong,
    title: "天选之子",
    tier: AchievementTier::Gold,
    description: "完成所有不低于64番的番种",
    criteria: &[
        AchievementCriterion {
            id: "BigFourWinds",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::BigFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "BigThreeDragons",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::BigThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllGreen",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::AllGreen)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "NineGates",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::NineGates)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "FourKongs",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::FourKongs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "SevenShiftedPairs",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::SevenShiftedPairs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "ThirteenOrphans",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::ThirteenOrphans)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllTerminals",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::AllTerminals)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "LittleFourWinds",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::LittleFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "LittleThreeDragons",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::LittleThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "AllHonors",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::AllHonors)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "FourConcealedPungs",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::FourConcealedPungs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        },
        AchievementCriterion {
            id: "PureTerminalChows",
            amount: |event| u64::from(mahjong_fan(event, MahjongFan::PureTerminalChows)),
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
