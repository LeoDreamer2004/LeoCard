//! Shared achievement registry. Add definitions here; gameplay emits facts, not awards.

use leocard_mahjong::Fan;
use leocard_protocol::{AchievementCounts, GameEvent, MahjongEvent, PlayerId, PlayerInteraction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AchievementTier {
    Bronze,
    Silver,
    Gold,
}
impl AchievementTier {
    pub const fn medal_index(self) -> usize {
        match self {
            Self::Gold => 0,
            Self::Silver => 1,
            Self::Bronze => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AchievementCategory {
    #[default]
    Mahjong,
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
    Personal,
}

/// Live facts emitted by game, social, or local systems. Snapshots are never triggers.
#[derive(Clone, Debug)]
pub enum AchievementTrigger {
    Game {
        player: PlayerId,
        event: GameEvent,
    },
    Interaction {
        player: PlayerId,
        event: PlayerInteraction,
    },
    Signal(&'static str),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AchievementScope {
    #[default]
    Lifetime,
    Match,
    Hand,
}

pub struct AchievementCriterion {
    pub id: &'static str,
    pub amount: fn(&AchievementTrigger) -> u64,
    pub target: u64,
    pub scope: AchievementScope,
}

pub struct AchievementDefinition {
    pub id: &'static str,
    pub category: AchievementCategory,
    pub title: &'static str,
    pub tier: AchievementTier,
    pub description: &'static str,
    pub criteria: &'static [AchievementCriterion],
    /// All groups must be satisfied; any criterion within a group suffices.
    pub requirements: &'static [&'static [&'static str]],
}

pub(super) fn mahjong_fan(trigger: &AchievementTrigger, fan: Fan) -> bool {
    let AchievementTrigger::Game {
        player,
        event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
    } = trigger
    else {
        return false;
    };
    result.winners.iter().any(|winner| {
        winner.player == *player
            && winner
                .score
                .fans
                .iter()
                .any(|value| value.fan == fan && value.count > 0)
    })
}

/// The only definition list used by the catalogue, trigger engine and medal totals.
pub static ACHIEVEMENT_REGISTRY: &[AchievementDefinition] = &[
    AchievementDefinition {
        id: "leocard:mahjong/big_four_winds",
        category: AchievementCategory::Mahjong,
        title: "风神召唤",
        tier: AchievementTier::Silver,
        description: "和出番种「大四喜」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::BigFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/big_three_dragons",
        category: AchievementCategory::Mahjong,
        title: "三位一体",
        tier: AchievementTier::Silver,
        description: "和出番种「大三元」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::BigThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/all_green",
        category: AchievementCategory::Mahjong,
        title: "绿满乾坤",
        tier: AchievementTier::Silver,
        description: "和出番种「绿一色」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllGreen)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/nine_gates",
        category: AchievementCategory::Mahjong,
        title: "莲灯九转",
        tier: AchievementTier::Silver,
        description: "和出番种「九莲宝灯」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::NineGates)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/four_kongs",
        category: AchievementCategory::Mahjong,
        title: "四柱擎天",
        tier: AchievementTier::Silver,
        description: "和出番种「四杠」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::FourKongs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/seven_shifted_pairs",
        category: AchievementCategory::Mahjong,
        title: "七星连珠",
        tier: AchievementTier::Silver,
        description: "和出番种「连七对」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::SevenShiftedPairs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/thirteen_orphans",
        category: AchievementCategory::Mahjong,
        title: "国士无双",
        tier: AchievementTier::Silver,
        description: "和出番种「十三幺」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::ThirteenOrphans)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/all_terminals",
        category: AchievementCategory::Mahjong,
        title: "九九归一",
        tier: AchievementTier::Silver,
        description: "和出番种「清幺九」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllTerminals)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/little_four_winds",
        category: AchievementCategory::Mahjong,
        title: "风神眷顾",
        tier: AchievementTier::Silver,
        description: "和出番种「小四喜」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::LittleFourWinds)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/little_three_dragons",
        category: AchievementCategory::Mahjong,
        title: "三元聚首",
        tier: AchievementTier::Silver,
        description: "和出番种「小三元」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::LittleThreeDragons)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/all_honors",
        category: AchievementCategory::Mahjong,
        title: "字字珠玑",
        tier: AchievementTier::Silver,
        description: "和出番种「字一色」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::AllHonors)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/four_concealed_pungs",
        category: AchievementCategory::Mahjong,
        title: "四暗藏锋",
        tier: AchievementTier::Silver,
        description: "和出番种「四暗刻」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::FourConcealedPungs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/pure_terminal_chows",
        category: AchievementCategory::Mahjong,
        title: "双龙戏珠",
        tier: AchievementTier::Silver,
        description: "和出番种「一色双龙会」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::PureTerminalChows)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/quadruple_chow",
        category: AchievementCategory::Mahjong,
        title: "四顺同源",
        tier: AchievementTier::Silver,
        description: "和出番种「一色四同顺」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::QuadrupleChow)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
    AchievementDefinition {
        id: "leocard:mahjong/four_pure_shifted_pungs",
        category: AchievementCategory::Mahjong,
        title: "节节高升",
        tier: AchievementTier::Silver,
        description: "和出番种「一色四节高」",
        criteria: &[AchievementCriterion {
            id: "win",
            amount: |event| u64::from(mahjong_fan(event, Fan::FourPureShiftedPungs)),
            target: 1,
            scope: AchievementScope::Lifetime,
        }],
        requirements: &[&["win"]],
    },
];

pub fn achievement_by_id(id: &str) -> Option<&'static AchievementDefinition> {
    ACHIEVEMENT_REGISTRY
        .iter()
        .find(|definition| definition.id == id)
}

pub fn achievements_in(
    category: AchievementCategory,
) -> impl Iterator<Item = &'static AchievementDefinition> {
    ACHIEVEMENT_REGISTRY
        .iter()
        .filter(move |definition| definition.category == category)
}

pub fn achievement_counts<'a>(ids: impl IntoIterator<Item = &'a str>) -> AchievementCounts {
    let mut seen = std::collections::HashSet::new();
    let mut counts = [0_u32; 3];
    for id in ids {
        if seen.insert(id)
            && let Some(definition) = achievement_by_id(id)
        {
            counts[definition.tier.medal_index()] += 1;
        }
    }
    AchievementCounts {
        gold: counts[0],
        silver: counts[1],
        bronze: counts[2],
    }
}
