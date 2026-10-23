use leocard_mahjong::Fan;

pub(super) struct FanGuideEntry {
    pub fan: Fan,
    pub requirement: &'static str,
    pub example: &'static str,
}

macro_rules! entry {
    ($fan:ident, $requirement:literal, $example:literal) => {
        FanGuideEntry {
            fan: Fan::$fan,
            requirement: $requirement,
            example: $example,
        }
    };
}

// 示例中的每组牌用空格分开；! 表示该组已经副露。情境番的牌面仅作示意。
pub(super) const ENTRIES: &[FanGuideEntry] = &[
    entry!(
        BigFourWinds,
        "东、南、西、北四组风刻（或杠），另有一对将牌。",
        "EEE SSS WWW NNN 11m"
    ),
    entry!(
        BigThreeDragons,
        "中、发、白三组箭刻（或杠）。",
        "RRR GGG HHH 123m 22m"
    ),
    entry!(
        AllGreen,
        "整副和牌仅由二、三、四、六、八条及发牌组成。",
        "222s 333s 444s 666s GG"
    ),
    entry!(
        NineGates,
        "门前清持有同一花色的 1112345678999，和同花色任意一张。",
        "1112345678999m 5m"
    ),
    entry!(
        FourKongs,
        "一副和牌中有四组杠。",
        "!1111m !2222p !3333s !EEEE 55m"
    ),
    entry!(
        SevenShiftedPairs,
        "门前清，由同一花色连续七个序数各组成一对。",
        "11m 22m 33m 44m 55m 66m 77m"
    ),
    entry!(
        ThirteenOrphans,
        "门前清，由三门的幺九牌、七种字牌各一张，再配其中任意一张作将。",
        "19m 19p 19s ESWNRGH E"
    ),
    entry!(
        AllTerminals,
        "只用一、九序数牌组成刻子（或杠）及将牌。",
        "111m 999m 111p 999p 11s"
    ),
    entry!(
        LittleFourWinds,
        "三组风刻（或杠），第四种风牌作将。",
        "EEE SSS WWW NN 123m"
    ),
    entry!(
        LittleThreeDragons,
        "两组箭刻（或杠），第三种箭牌作将。",
        "RRR GGG HH 123m 456m"
    ),
    entry!(
        AllHonors,
        "和牌全部由风牌、箭牌组成，且为刻子（或杠）与将牌。",
        "EEE SSS RRR GGG HH"
    ),
    entry!(
        FourConcealedPungs,
        "四组暗刻（暗杠）和一对将牌。",
        "111m 222p 333s EEE 55m"
    ),
    entry!(
        PureTerminalChows,
        "同一花色的两组 123、两组 789，及一对 5 作将。",
        "123m 123m 789m 789m 55m"
    ),
    entry!(
        QuadrupleChow,
        "同一花色、相同数字的顺子共四组。",
        "123m 123m 123m 123m 55m"
    ),
    entry!(
        FourPureShiftedPungs,
        "同一花色的四组刻子（或杠），数字逐组递增一位。",
        "111m 222m 333m 444m 55m"
    ),
    entry!(
        FourPureShiftedChows,
        "同一花色的四组顺子，起点逐组递增一位或两位。",
        "123m 234m 345m 456m 77m"
    ),
    entry!(
        ThreeKongs,
        "一副和牌中有三组杠。",
        "!1111m !2222p !3333s 456m 77m"
    ),
    entry!(
        AllTerminalsAndHonors,
        "仅用幺九牌和字牌组成刻子（或杠）及将牌。",
        "111m 999p EEE RRR 11s"
    ),
    entry!(
        SevenPairs,
        "门前清，由七个对子组成。",
        "11m 22m 33p 44p 55s 66s EE"
    ),
    entry!(
        GreaterHonorsAndKnittedTiles,
        "七种字牌各一张，另有七张来自三色错位的 147、258、369 序数牌；不得重复。",
        "147m 258p 3s ESWNRGH"
    ),
    entry!(
        AllEvenPungs,
        "仅用二、四、六、八序数牌组成刻子（或杠）与将牌。",
        "222m 444p 666s 888m 22p"
    ),
    entry!(
        FullFlush,
        "和牌只使用同一花色的序数牌。",
        "123m 345m 567m 789m 99m"
    ),
    entry!(
        PureTripleChow,
        "同一花色、相同数字的顺子共三组。",
        "123m 123m 123m 456m 77m"
    ),
    entry!(
        PureShiftedPungs,
        "同一花色三组刻子（或杠），数字逐组递增一位。",
        "222m 333m 444m 567p 88p"
    ),
    entry!(
        UpperTiles,
        "整副和牌的序数牌只使用七、八、九。",
        "789m 789p 777s 888m 99p"
    ),
    entry!(
        MiddleTiles,
        "整副和牌的序数牌只使用四、五、六。",
        "456m 456p 444s 555m 66p"
    ),
    entry!(
        LowerTiles,
        "整副和牌的序数牌只使用一、二、三。",
        "123m 123p 111s 222m 33p"
    ),
    entry!(
        PureStraight,
        "同一花色由 123、456、789 三组顺子连成一至九。",
        "123m 456m 789m 111p 22p"
    ),
    entry!(
        ThreeSuitedTerminalChows,
        "两个花色各有一组 123 和 789，第三花色的 5 作将。",
        "123m 789m 123p 789p 55s"
    ),
    entry!(
        PureShiftedChows,
        "同一花色三组顺子，起点逐组递增一位或两位。",
        "123m 234m 345m 789p 77p"
    ),
    entry!(
        AllFives,
        "四组面子和将牌，每组都含有序数五。",
        "345m 456p 555s 567m 55p"
    ),
    entry!(
        TriplePung,
        "三个花色中，相同数字的刻子（或杠）各一组。",
        "222m 222p 222s 345m 77m"
    ),
    entry!(
        ThreeConcealedPungs,
        "一副和牌中有三组暗刻（暗杠）。",
        "111m 222p 333s !456m 77p"
    ),
    entry!(
        LesserHonorsAndKnittedTiles,
        "三门错位的 147、258、369 序数牌加字牌，凑足十四张且不得重复。",
        "147m 258p 369s ESWNR"
    ),
    entry!(
        KnittedStraight,
        "三门分别取 147、258、369，组成九张不重复的组合龙。",
        "147m 258p 369s 111m 22p"
    ),
    entry!(
        UpperFour,
        "和牌仅用六、七、八、九序数牌。",
        "678m 789m 777p 888s 99p"
    ),
    entry!(
        LowerFour,
        "和牌仅用一、二、三、四序数牌。",
        "123m 234m 111p 222s 33p"
    ),
    entry!(
        BigThreeWinds,
        "三种风牌各组成一组刻子（或杠）。",
        "EEE SSS WWW 123m 55m"
    ),
    entry!(
        MixedStraight,
        "三个花色分别组成 123、456、789，连成一至九。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        ReversibleTiles,
        "和牌只由上下颠倒后图案仍相同的牌组成。",
        "222p 444p 888p HHH 55p"
    ),
    entry!(
        MixedTripleChow,
        "三个花色中，相同数字的顺子各一组。",
        "123m 123p 123s 456m 77m"
    ),
    entry!(
        MixedShiftedPungs,
        "三个花色的刻子（或杠），数字逐组递增一位。",
        "222m 333p 444s 567m 88p"
    ),
    entry!(
        ChickenHand,
        "和牌不符合其他任何番种（花牌另计）；仍须以实际计分结果为准。",
        "!234m 456p 678s 222s EE"
    ),
    entry!(
        LastTileDraw,
        "自摸牌墙最后一张牌和牌。牌型仅作示意。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        LastTileClaim,
        "和他人打出的本局最后一张牌。牌型仅作示意。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        OutWithReplacementTile,
        "开杠后，用杠上补得的牌自摸和牌；补花所得不算。",
        "!1111m 234p 567s 789m 22p"
    ),
    entry!(
        RobbingTheKong,
        "和他人加杠时所加的那张牌。牌型仅作示意。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        TwoConcealedKongs,
        "一副和牌中有两组暗杠。",
        "1111m 2222p 345s 678s 99m"
    ),
    entry!(
        AllPungs,
        "四组刻子（或杠）加一对将牌，没有顺子。",
        "111m 222p 333s EEE 55m"
    ),
    entry!(
        HalfFlush,
        "仅使用同一花色的序数牌及字牌。",
        "123m 456m 789m EEE RR"
    ),
    entry!(
        MixedShiftedChows,
        "三个花色的顺子，起点逐组递增一位。",
        "123m 234p 345s 789m 77p"
    ),
    entry!(
        AllTypes,
        "和牌包含万、条、饼、风牌、箭牌五类牌。",
        "123m 456p 789s EEE RR"
    ),
    entry!(
        MeldedHand,
        "四组面子均由吃、碰或明杠形成，最后单钓他人打出的将牌和牌。",
        "!123m !456p !789s !EEE 55m"
    ),
    entry!(
        TwoDragonPungs,
        "中、发、白之中两种箭牌各组成一组刻子（或杠）。",
        "RRR GGG 123m 456p 77s"
    ),
    entry!(
        OutsideHand,
        "四组面子和将牌各含幺九牌或字牌。",
        "123m 789p 111s EEE 99p"
    ),
    entry!(
        FullyConcealedHand,
        "没有吃、碰、明杠，且以自摸和牌。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        TwoMeldedKongs,
        "一副和牌中有两组明杠。明杠与暗杠各一组算 6 番。",
        "!1111m !2222p 345s 678s 99m"
    ),
    entry!(
        LastTile,
        "和某种牌的第四张；前三张已在弃牌或副露中亮明。",
        "111m 234p 567s 789m 22p"
    ),
    entry!(
        DragonPung,
        "中、发、白任一种箭牌组成刻子（或杠）。",
        "RRR 123m 456p 789s 22m"
    ),
    entry!(
        PrevalentWind,
        "本圈的圈风牌组成刻子（或杠）；示例假设东风圈。",
        "EEE 123m 456p 789s 22m"
    ),
    entry!(
        SeatWind,
        "本人的门风牌组成刻子（或杠）；示例假设东家。",
        "EEE 123m 456p 789s 22m"
    ),
    entry!(
        ConcealedHand,
        "没有吃、碰、明杠，以他人打出的牌和牌。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        AllChows,
        "四组顺子加一对序数牌作将，且全牌无字牌。",
        "123m 456p 789s 234m 55p"
    ),
    entry!(
        TileHog,
        "四张相同的牌都进入和牌，但不组成杠。",
        "111m 123m 456p 789s 22p"
    ),
    entry!(
        DoublePung,
        "两个花色中，相同数字的刻子（或杠）各一组。",
        "222m 222p 345s 678m 99p"
    ),
    entry!(
        TwoConcealedPungs,
        "一副和牌中有两组暗刻（暗杠）。",
        "111m 222p !345s !678m 99m"
    ),
    entry!(
        ConcealedKong,
        "手中四张相同牌开暗杠，不取他人弃牌。",
        "1111m 234p 567s 789m 22p"
    ),
    entry!(
        AllSimples,
        "和牌中没有一、九序数牌和字牌。",
        "234m 345p 456s 678m 22p"
    ),
    entry!(
        PureDoubleChow,
        "同一花色、相同数字的顺子两组。",
        "123m 123m 456p 789s 22p"
    ),
    entry!(
        MixedDoubleChow,
        "两个花色中，相同数字的顺子各一组。",
        "123m 123p 456s 789m 22p"
    ),
    entry!(
        ShortStraight,
        "同一花色的两组顺子相连，组成连续六张序数牌。",
        "123m 456m 789p 111s 22s"
    ),
    entry!(
        TwoTerminalChows,
        "同一花色各有一组 123 和 789。",
        "123m 789m 456p 111s 22s"
    ),
    entry!(
        PungOfTerminalsOrHonors,
        "幺九序数牌或风牌组成一组刻子（或杠）。",
        "111m 123p 456s 789m 22p"
    ),
    entry!(
        MeldedKong,
        "取他人弃牌开杠，或将已碰的牌加杠。",
        "!1111m 234p 567s 789m 22p"
    ),
    entry!(
        OneVoidedSuit,
        "和牌只含三种花色中的两种，可以有字牌。",
        "123m 456p 789m 111p 22m"
    ),
    entry!(
        NoHonors,
        "和牌中没有任何风牌或箭牌。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        EdgeWait,
        "以 12 等 3，或以 89 等 7 的唯一和牌张；示例为等 3 万。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        ClosedWait,
        "顺子中间缺一张的唯一和牌张；示例为 24 万等 3 万。",
        "234m 456p 789s 111m 22p"
    ),
    entry!(
        SingleWait,
        "单张等将牌的唯一和牌张；示例为等 2 饼。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        SelfDrawn,
        "自己从牌墙摸到和牌张；不包括和他人打出的牌。",
        "123m 456p 789s 111m 22p"
    ),
    entry!(
        FlowerTiles,
        "每张春、夏、秋、冬、梅、兰、竹、菊花牌各计一番；花牌不计入起和番数。",
        "a b c d e f g h"
    ),
];
