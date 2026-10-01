//! 全局成就注册表，图鉴、求值、计数和公告共用。
use super::mahjong::*;
use super::types::*;
use leocard_mahjong::{Fan, MahjongMatchLength};
use leocard_protocol::AchievementCounts;
use std::collections::HashSet;

macro_rules! award {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr, $scope:ident) => {
        AchievementDefinition {
            id: concat!("leocard:mahjong/", $id),
            category: AchievementCategory::Mahjong,
            title: $title,
            tier: AchievementTier::$tier,
            description: $description,
            criteria: &[AchievementCriterion {
                id: "progress",
                amount: $amount,
                target: $target,
                scope: AchievementScope::$scope,
            }],
            requirements: &[&["progress"]],
        }
    };
}
macro_rules! fan {
    ($id:literal, $title:literal, $tier:ident, $fan:ident, $description:literal) => {
        award!(
            $id,
            $title,
            $tier,
            $description,
            |event| u64::from(mahjong_fan(event, Fan::$fan)),
            1,
            Lifetime
        )
    };
}

pub static ACHIEVEMENT_REGISTRY: &[AchievementDefinition] = &[
    fan!(
        "big_four_winds",
        "风神召唤",
        Silver,
        BigFourWinds,
        "和出番种「大四喜」"
    ),
    fan!(
        "big_three_dragons",
        "三位一体",
        Silver,
        BigThreeDragons,
        "和出番种「大三元」"
    ),
    fan!(
        "all_green",
        "绿满乾坤",
        Silver,
        AllGreen,
        "和出番种「绿一色」"
    ),
    fan!(
        "nine_gates",
        "莲灯九转",
        Silver,
        NineGates,
        "和出番种「九莲宝灯」"
    ),
    fan!(
        "four_kongs",
        "四柱擎天",
        Silver,
        FourKongs,
        "和出番种「四杠」"
    ),
    fan!(
        "seven_shifted_pairs",
        "七星连珠",
        Silver,
        SevenShiftedPairs,
        "和出番种「连七对」"
    ),
    fan!(
        "thirteen_orphans",
        "国士无双",
        Silver,
        ThirteenOrphans,
        "和出番种「十三幺」"
    ),
    fan!(
        "all_terminals",
        "九九归一",
        Silver,
        AllTerminals,
        "和出番种「清幺九」"
    ),
    fan!(
        "little_four_winds",
        "风神眷顾",
        Silver,
        LittleFourWinds,
        "和出番种「小四喜」"
    ),
    fan!(
        "little_three_dragons",
        "三元聚首",
        Silver,
        LittleThreeDragons,
        "和出番种「小三元」"
    ),
    fan!(
        "all_honors",
        "字字珠玑",
        Silver,
        AllHonors,
        "和出番种「字一色」"
    ),
    fan!(
        "four_concealed_pungs",
        "四暗藏锋",
        Silver,
        FourConcealedPungs,
        "和出番种「四暗刻」"
    ),
    fan!(
        "pure_terminal_chows",
        "双龙戏珠",
        Silver,
        PureTerminalChows,
        "和出番种「一色双龙会」"
    ),
    fan!(
        "quadruple_chow",
        "四顺同源",
        Silver,
        QuadrupleChow,
        "和出番种「一色四同顺」"
    ),
    fan!(
        "four_pure_shifted_pungs",
        "节节高升",
        Silver,
        FourPureShiftedPungs,
        "和出番种「一色四节高」"
    ),
    fan!(
        "mixed_shifted_chows",
        "初窥门径",
        Bronze,
        MixedShiftedChows,
        "和出番种「三色三步高」"
    ),
    fan!(
        "mixed_triple_chow",
        "读起来就是朗朗上口",
        Bronze,
        MixedTripleChow,
        "和出番种「三色三同顺」"
    ),
    fan!(
        "mixed_straight",
        "龙飞凤舞",
        Bronze,
        MixedStraight,
        "和出番种「花龙」"
    ),
    fan!(
        "pure_straight",
        "一气呵成",
        Bronze,
        PureStraight,
        "和出番种「清龙」"
    ),
    fan!(
        "half_flush",
        "从左往右打就可以了",
        Bronze,
        HalfFlush,
        "和出番种「混一色」"
    ),
    fan!(
        "pure_shifted_chows",
        "没有那么难呢",
        Bronze,
        PureShiftedChows,
        "和出番种「一色三步高」"
    ),
    fan!(
        "all_types",
        "五谷丰登",
        Bronze,
        AllTypes,
        "和出番种「五门齐」"
    ),
    fan!(
        "all_pungs",
        "有碰才有杠",
        Bronze,
        AllPungs,
        "和出番种「碰碰和」"
    ),
    fan!(
        "seven_pairs",
        "有碰也不碰",
        Bronze,
        SevenPairs,
        "和出番种「七对」"
    ),
    fan!(
        "outside_hand",
        "拖衣带水",
        Bronze,
        OutsideHand,
        "和出番种「全带幺」"
    ),
    fan!(
        "knitted_straight",
        "奇怪的组合",
        Bronze,
        KnittedStraight,
        "和出番种「组合龙」"
    ),
    fan!(
        "lesser_honors_knitted",
        "破烂手牌的出路",
        Bronze,
        LesserHonorsAndKnittedTiles,
        "和出番种「全不靠」"
    ),
    fan!(
        "upper_four",
        "我喜欢大的",
        Bronze,
        UpperFour,
        "和出番种「大于五」"
    ),
    fan!(
        "lower_four",
        "我喜欢小的",
        Bronze,
        LowerFour,
        "和出番种「小于五」"
    ),
    fan!(
        "chicken_hand",
        "无中生有",
        Bronze,
        ChickenHand,
        "和出番种「无番和」"
    ),
    fan!(
        "melded_hand",
        "不能没有你",
        Bronze,
        MeldedHand,
        "和出番种「全求人」"
    ),
    fan!(
        "robbing_kong",
        "那个杠不成立！",
        Bronze,
        RobbingTheKong,
        "和出番种「抢杠和」"
    ),
    fan!(
        "replacement_win",
        "高岭之花",
        Bronze,
        OutWithReplacementTile,
        "和出番种「杠上开花」"
    ),
    fan!(
        "last_tile_claim",
        "笑到最后",
        Bronze,
        LastTileClaim,
        "和出番种「海底捞月」"
    ),
    award!(
        "small_fan_collector",
        "总之就是凑数的",
        Bronze,
        "和出1～2番的番种共200番次（不计花牌）",
        mahjong_low_fans,
        200,
        Lifetime
    ),
    award!(
        "false_win",
        "这是为什么",
        Bronze,
        "错和一次",
        mahjong_false_win,
        1,
        Lifetime
    ),
    award!(
        "first_ten_wins",
        "小试牛刀",
        Bronze,
        "和牌10次",
        mahjong_wins,
        10,
        Lifetime
    ),
    award!(
        "big_eight_collector",
        "萌新之友",
        Bronze,
        "和出国标八大番之一共50次「三色三步高」、「三色三同顺」、「花龙」、「清龙」、「混一色」、「一色三步高」、「五门齐」、「碰碰和」",
        |event| mahjong_group(event, BIG_EIGHT),
        50,
        Lifetime
    ),
    award!(
        "small_eight_collector",
        "更进一步",
        Bronze,
        "和出国标八小番之一共50次「七对」、「全带幺」、「组合龙」、「全不靠」、「大于五」、「小于五」、「无番和」、「全求人」",
        |event| mahjong_group(event, SMALL_EIGHT),
        50,
        Lifetime
    ),
    award!(
        "almost_major_collector",
        "寸止的艺术",
        Bronze,
        "和出以下番种之一共50次「三暗刻」「三杠」「混幺九」",
        |event| mahjong_group(
            event,
            &[
                Fan::ThreeConcealedPungs,
                Fan::ThreeKongs,
                Fan::AllTerminalsAndHonors
            ]
        ),
        50,
        Lifetime
    ),
    award!(
        "last_hand_comeback",
        "永不言弃",
        Bronze,
        "在一场游戏的最后一局，从第四名翻盘到第一名",
        |event| u64::from(
            MahjongFacts::from_trigger(event).is_some_and(|facts| facts.last_hand_comeback())
        ),
        1,
        Lifetime
    ),
    award!(
        "six_fan_eight_points",
        "我是怎么胡的？",
        Bronze,
        "在拥有至少6种不同番种的情况下，恰好达到8番和牌条件（不计花牌）",
        |event| mahjong_eight_points(event, false),
        1,
        Lifetime
    ),
    award!(
        "five_hundred_wins",
        "千锤百炼",
        Silver,
        "和牌500次",
        mahjong_wins,
        500,
        Lifetime
    ),
    award!(
        "three_winner_discard",
        "三家分晋",
        Silver,
        "在启用一炮多响的一局中给三家同时点炮",
        |event| u64::from(MahjongFacts::from_trigger(event).is_some_and(|facts| facts.fed_three())),
        1,
        Lifetime
    ),
    award!(
        "east_round_300",
        "东风浩荡",
        Silver,
        "在一场东风局游戏中，获得300分以上",
        |event| u64::from(
            MahjongFacts::from_trigger(event)
                .is_some_and(|facts| facts.final_score(MahjongMatchLength::EastRound, 300))
        ),
        1,
        Lifetime
    ),
    award!(
        "south_round_500",
        "南风送爽",
        Silver,
        "在一场南风局游戏中，获得500分以上",
        |event| u64::from(
            MahjongFacts::from_trigger(event)
                .is_some_and(|facts| facts.final_score(MahjongMatchLength::HalfGame, 500))
        ),
        1,
        Lifetime
    ),
    award!(
        "eight_one_point_fans",
        "一切尽在掌握之中",
        Silver,
        "通过8种1番的番种，恰好达到8番和牌条件（不计花牌）",
        |event| mahjong_eight_points(event, true),
        1,
        Lifetime
    ),
    award!(
        "ten_thousand_hands",
        "万里挑一",
        Gold,
        "累计完成10000局，包含流局；多局游戏中的每局分别计数",
        mahjong_hands,
        10000,
        Lifetime
    ),
    award!(
        "two_major_fans",
        "龙凤胎",
        Gold,
        "一次性和出至少两个不低于48番的番种",
        |event| u64::from(
            MahjongFacts::from_trigger(event)
                .is_some_and(|facts| facts.unique_fans(|fan| fan.points() >= 48) >= 2)
        ),
        1,
        Lifetime
    ),
    award!(
        "two_major_wins",
        "双喜临门",
        Gold,
        "在一场多局游戏中，至少两次和牌各含一个不低于48番的番种",
        mahjong_high_win,
        2,
        Match
    ),
    award!(
        "full_game_700",
        "威震华夏",
        Gold,
        "在一场全庄局游戏中，获得700分以上",
        |event| u64::from(
            MahjongFacts::from_trigger(event)
                .is_some_and(|facts| facts.final_score(MahjongMatchLength::FullGame, 700))
        ),
        1,
        Lifetime
    ),
    award!(
        "broad_full_flush",
        "搞不懂在听什么牌",
        Bronze,
        "在拥有至少5种听牌的情况下和出「清一色」",
        |event| u64::from(
            MahjongFacts::from_trigger(event).is_some_and(|facts| facts.broad_flush())
        ),
        1,
        Lifetime
    ),
    award!(
        "all_draw_match",
        "实属煎熬",
        Silver,
        "在一场多局游戏中，以所有局全部流局结束",
        |event| u64::from(MahjongFacts::from_trigger(event).is_some_and(|facts| facts.all_draws())),
        1,
        Lifetime
    ),
    award!(
        "six_insufficient_fans",
        "爱而不得",
        Bronze,
        "在一局中，至少6次因为番种不够而无法和牌",
        mahjong_insufficient_fan,
        6,
        Hand
    ),
    AchievementDefinition {
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
    let mut seen = HashSet::new();
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
