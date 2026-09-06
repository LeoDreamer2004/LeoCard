#[path = "scoring_forms.rs"]
mod forms;
#[path = "scoring_patterns.rs"]
mod patterns;
#[cfg(test)]
#[path = "scoring_tests.rs"]
mod tests;

use crate::{
    MahjongDragon, MahjongKongKind, MahjongMeldKind, MahjongPlayerId, MahjongSuit, MahjongTileKind,
    MahjongWind, Meld,
};
use forms::{unique_wait, validated_forms};
use patterns::*;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fan {
    BigFourWinds,
    BigThreeDragons,
    AllGreen,
    NineGates,
    FourKongs,
    SevenShiftedPairs,
    ThirteenOrphans,
    AllTerminals,
    LittleFourWinds,
    LittleThreeDragons,
    AllHonors,
    FourConcealedPungs,
    PureTerminalChows,
    QuadrupleChow,
    FourPureShiftedPungs,
    FourPureShiftedChows,
    ThreeKongs,
    AllTerminalsAndHonors,
    SevenPairs,
    GreaterHonorsAndKnittedTiles,
    AllEvenPungs,
    FullFlush,
    PureTripleChow,
    PureShiftedPungs,
    UpperTiles,
    MiddleTiles,
    LowerTiles,
    PureStraight,
    ThreeSuitedTerminalChows,
    PureShiftedChows,
    AllFives,
    TriplePung,
    ThreeConcealedPungs,
    LesserHonorsAndKnittedTiles,
    KnittedStraight,
    UpperFour,
    LowerFour,
    BigThreeWinds,
    MixedStraight,
    ReversibleTiles,
    MixedTripleChow,
    MixedShiftedPungs,
    ChickenHand,
    LastTileDraw,
    LastTileClaim,
    OutWithReplacementTile,
    RobbingTheKong,
    TwoConcealedKongs,
    AllPungs,
    HalfFlush,
    MixedShiftedChows,
    AllTypes,
    MeldedHand,
    TwoDragonPungs,
    OutsideHand,
    FullyConcealedHand,
    TwoMeldedKongs,
    LastTile,
    DragonPung,
    PrevalentWind,
    SeatWind,
    ConcealedHand,
    AllChows,
    TileHog,
    DoublePung,
    TwoConcealedPungs,
    ConcealedKong,
    AllSimples,
    PureDoubleChow,
    MixedDoubleChow,
    ShortStraight,
    TwoTerminalChows,
    PungOfTerminalsOrHonors,
    MeldedKong,
    OneVoidedSuit,
    NoHonors,
    EdgeWait,
    ClosedWait,
    SingleWait,
    SelfDrawn,
    FlowerTiles,
}

impl Fan {
    pub const fn points(self) -> u16 {
        match self {
            Self::BigFourWinds
            | Self::BigThreeDragons
            | Self::AllGreen
            | Self::NineGates
            | Self::FourKongs
            | Self::SevenShiftedPairs
            | Self::ThirteenOrphans => 88,
            Self::AllTerminals
            | Self::LittleFourWinds
            | Self::LittleThreeDragons
            | Self::AllHonors
            | Self::FourConcealedPungs
            | Self::PureTerminalChows => 64,
            Self::QuadrupleChow | Self::FourPureShiftedPungs => 48,
            Self::FourPureShiftedChows | Self::ThreeKongs | Self::AllTerminalsAndHonors => 32,
            Self::SevenPairs
            | Self::GreaterHonorsAndKnittedTiles
            | Self::AllEvenPungs
            | Self::FullFlush
            | Self::PureTripleChow
            | Self::PureShiftedPungs
            | Self::UpperTiles
            | Self::MiddleTiles
            | Self::LowerTiles => 24,
            Self::PureStraight
            | Self::ThreeSuitedTerminalChows
            | Self::PureShiftedChows
            | Self::AllFives
            | Self::TriplePung
            | Self::ThreeConcealedPungs => 16,
            Self::LesserHonorsAndKnittedTiles
            | Self::KnittedStraight
            | Self::UpperFour
            | Self::LowerFour
            | Self::BigThreeWinds => 12,
            Self::MixedStraight
            | Self::ReversibleTiles
            | Self::MixedTripleChow
            | Self::MixedShiftedPungs
            | Self::ChickenHand
            | Self::LastTileDraw
            | Self::LastTileClaim
            | Self::OutWithReplacementTile
            | Self::RobbingTheKong
            | Self::TwoConcealedKongs => 8,
            Self::AllPungs
            | Self::HalfFlush
            | Self::MixedShiftedChows
            | Self::AllTypes
            | Self::MeldedHand
            | Self::TwoDragonPungs => 6,
            Self::OutsideHand
            | Self::FullyConcealedHand
            | Self::TwoMeldedKongs
            | Self::LastTile => 4,
            Self::DragonPung
            | Self::PrevalentWind
            | Self::SeatWind
            | Self::ConcealedHand
            | Self::AllChows
            | Self::TileHog
            | Self::DoublePung
            | Self::TwoConcealedPungs
            | Self::ConcealedKong
            | Self::AllSimples => 2,
            Self::PureDoubleChow
            | Self::MixedDoubleChow
            | Self::ShortStraight
            | Self::TwoTerminalChows
            | Self::PungOfTerminalsOrHonors
            | Self::MeldedKong
            | Self::OneVoidedSuit
            | Self::NoHonors
            | Self::EdgeWait
            | Self::ClosedWait
            | Self::SingleWait
            | Self::SelfDrawn
            | Self::FlowerTiles => 1,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::BigFourWinds => "大四喜",
            Self::BigThreeDragons => "大三元",
            Self::AllGreen => "绿一色",
            Self::NineGates => "九莲宝灯",
            Self::FourKongs => "四杠",
            Self::SevenShiftedPairs => "连七对",
            Self::ThirteenOrphans => "十三幺",
            Self::AllTerminals => "清幺九",
            Self::LittleFourWinds => "小四喜",
            Self::LittleThreeDragons => "小三元",
            Self::AllHonors => "字一色",
            Self::FourConcealedPungs => "四暗刻",
            Self::PureTerminalChows => "一色双龙会",
            Self::QuadrupleChow => "一色四同顺",
            Self::FourPureShiftedPungs => "一色四节高",
            Self::FourPureShiftedChows => "一色四步高",
            Self::ThreeKongs => "三杠",
            Self::AllTerminalsAndHonors => "混幺九",
            Self::SevenPairs => "七对",
            Self::GreaterHonorsAndKnittedTiles => "七星不靠",
            Self::AllEvenPungs => "全双刻",
            Self::FullFlush => "清一色",
            Self::PureTripleChow => "一色三同顺",
            Self::PureShiftedPungs => "一色三节高",
            Self::UpperTiles => "全大",
            Self::MiddleTiles => "全中",
            Self::LowerTiles => "全小",
            Self::PureStraight => "清龙",
            Self::ThreeSuitedTerminalChows => "三色双龙会",
            Self::PureShiftedChows => "一色三步高",
            Self::AllFives => "全带五",
            Self::TriplePung => "三同刻",
            Self::ThreeConcealedPungs => "三暗刻",
            Self::LesserHonorsAndKnittedTiles => "全不靠",
            Self::KnittedStraight => "组合龙",
            Self::UpperFour => "大于五",
            Self::LowerFour => "小于五",
            Self::BigThreeWinds => "三风刻",
            Self::MixedStraight => "花龙",
            Self::ReversibleTiles => "推不倒",
            Self::MixedTripleChow => "三色三同顺",
            Self::MixedShiftedPungs => "三色三节高",
            Self::ChickenHand => "无番和",
            Self::LastTileDraw => "妙手回春",
            Self::LastTileClaim => "海底捞月",
            Self::OutWithReplacementTile => "杠上开花",
            Self::RobbingTheKong => "抢杠和",
            Self::TwoConcealedKongs => "双暗杠",
            Self::AllPungs => "碰碰和",
            Self::HalfFlush => "混一色",
            Self::MixedShiftedChows => "三色三步高",
            Self::AllTypes => "五门齐",
            Self::MeldedHand => "全求人",
            Self::TwoDragonPungs => "双箭刻",
            Self::OutsideHand => "全带幺",
            Self::FullyConcealedHand => "不求人",
            Self::TwoMeldedKongs => "双明杠",
            Self::LastTile => "和绝张",
            Self::DragonPung => "箭刻",
            Self::PrevalentWind => "圈风刻",
            Self::SeatWind => "门风刻",
            Self::ConcealedHand => "门前清",
            Self::AllChows => "平和",
            Self::TileHog => "四归一",
            Self::DoublePung => "双同刻",
            Self::TwoConcealedPungs => "双暗刻",
            Self::ConcealedKong => "暗杠",
            Self::AllSimples => "断幺",
            Self::PureDoubleChow => "一般高",
            Self::MixedDoubleChow => "喜相逢",
            Self::ShortStraight => "连六",
            Self::TwoTerminalChows => "老少副",
            Self::PungOfTerminalsOrHonors => "幺九刻",
            Self::MeldedKong => "明杠",
            Self::OneVoidedSuit => "缺一门",
            Self::NoHonors => "无字",
            Self::EdgeWait => "边张",
            Self::ClosedWait => "坎张",
            Self::SingleWait => "单调将",
            Self::SelfDrawn => "自摸",
            Self::FlowerTiles => "花牌",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FanValue {
    pub fan: Fan,
    pub count: u8,
    pub points: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WinSource {
    SelfDraw,
    Discard(MahjongPlayerId),
    KongReplacement,
    FlowerReplacement,
    RobbingKong(MahjongPlayerId),
}

impl WinSource {
    pub const fn is_self_draw(self) -> bool {
        matches!(
            self,
            Self::SelfDraw | Self::KongReplacement | Self::FlowerReplacement
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinContext {
    pub source: WinSource,
    pub seat_wind: MahjongWind,
    pub prevalent_wind: MahjongWind,
    pub last_wall_tile: bool,
    pub last_of_kind: bool,
    pub flower_count: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreInput {
    /// 包含和牌张的立牌，不包含副露与花牌。
    pub concealed: Vec<MahjongTileKind>,
    pub melds: Vec<Meld>,
    pub winning_tile: MahjongTileKind,
    pub context: WinContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongScoreResult {
    pub fans: Vec<FanValue>,
    /// 不含花牌的番数，用于判断是否达到 8 番起和。
    pub points_without_flowers: u16,
    pub flower_points: u8,
    pub total_points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScoreError {
    FlowerInHand,
    InvalidTileCount,
    TooManyCopies(MahjongTileKind),
    WinningTileMissing,
    NotComplete,
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlowerInHand => f.write_str("花牌必须先补花，不能留在和牌手牌中"),
            Self::InvalidTileCount => f.write_str("立牌与副露不能组成 14 张标准手牌"),
            Self::TooManyCopies(tile) => write!(f, "牌张数量超过四张：{tile}"),
            Self::WinningTileMissing => f.write_str("立牌中没有指定的和牌张"),
            Self::NotComplete => f.write_str("手牌不是合法和牌结构"),
        }
    }
}

impl std::error::Error for ScoreError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SetKind {
    Chow { suit: MahjongSuit, start: u8 },
    Pung(MahjongTileKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Set {
    kind: SetKind,
    open: bool,
    kong: Option<MahjongKongKind>,
}

#[derive(Clone, Debug)]
enum Form {
    Standard {
        sets: Vec<Set>,
        pair: MahjongTileKind,
    },
    SevenPairs {
        shifted: bool,
    },
    ThirteenOrphans,
    Knitted {
        greater: bool,
        straight: bool,
        sets: Vec<Set>,
        pair: Option<MahjongTileKind>,
    },
}

pub fn is_complete_hand(concealed: &[MahjongTileKind], melds: &[Meld]) -> bool {
    validated_forms(concealed, melds, None).is_ok_and(|forms| !forms.is_empty())
}

pub fn score_hand(input: &ScoreInput) -> Result<MahjongScoreResult, ScoreError> {
    let forms = validated_forms(&input.concealed, &input.melds, Some(input.winning_tile))?;
    if forms.is_empty() {
        return Err(ScoreError::NotComplete);
    }
    let unique_wait = unique_wait(&input.concealed, &input.melds, input.winning_tile);
    forms
        .iter()
        .map(|form| score_form(input, form, unique_wait))
        .max_by_key(|result| result.total_points)
        .ok_or(ScoreError::NotComplete)
}

fn score_form(input: &ScoreInput, form: &Form, unique_wait: bool) -> MahjongScoreResult {
    let mut values = BTreeMap::<Fan, (u8, u16)>::new();
    let add = |values: &mut BTreeMap<Fan, (u8, u16)>, fan: Fan, count: u8| {
        if count > 0 {
            values.insert(fan, (count, fan.points() * u16::from(count)));
        }
    };
    let all_tiles = all_tile_kinds(input);
    let counts = kind_counts(&all_tiles);
    let sets_pair = match form {
        Form::Standard { sets, pair } => Some((sets.as_slice(), *pair)),
        Form::Knitted {
            sets,
            pair: Some(pair),
            ..
        } => Some((sets.as_slice(), *pair)),
        _ => None,
    };
    let is_standard = matches!(form, Form::Standard { .. });
    let pungs: Vec<MahjongTileKind> = sets_pair
        .map(|(sets, _)| {
            sets.iter()
                .filter_map(|set| match set.kind {
                    SetKind::Pung(tile) => Some(tile),
                    SetKind::Chow { .. } => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let chows: Vec<(MahjongSuit, u8)> = sets_pair
        .map(|(sets, _)| {
            sets.iter()
                .filter_map(|set| match set.kind {
                    SetKind::Chow { suit, start } => Some((suit, start)),
                    SetKind::Pung(_) => None,
                })
                .collect()
        })
        .unwrap_or_default();

    let wind_pungs = pungs
        .iter()
        .filter(|tile| matches!(tile, MahjongTileKind::Wind(_)))
        .count();
    let dragon_pungs = pungs
        .iter()
        .filter(|tile| matches!(tile, MahjongTileKind::Dragon(_)))
        .count();
    let big_four = wind_pungs == 4;
    let big_three_dragons = dragon_pungs == 3;
    let little_four = sets_pair
        .is_some_and(|(_, pair)| wind_pungs == 3 && matches!(pair, MahjongTileKind::Wind(_)));
    let little_three = sets_pair
        .is_some_and(|(_, pair)| dragon_pungs == 2 && matches!(pair, MahjongTileKind::Dragon(_)));
    add(&mut values, Fan::BigFourWinds, u8::from(big_four));
    add(
        &mut values,
        Fan::BigThreeDragons,
        u8::from(big_three_dragons),
    );

    let all_green = all_tiles.iter().all(|tile| {
        matches!(
            tile,
            MahjongTileKind::Suited {
                suit: MahjongSuit::Bamboo,
                rank: 2 | 3 | 4 | 6 | 8
            } | MahjongTileKind::Dragon(MahjongDragon::Green)
        )
    });
    add(&mut values, Fan::AllGreen, u8::from(all_green));
    add(&mut values, Fan::NineGates, u8::from(is_nine_gates(input)));

    let kongs: Vec<MahjongKongKind> = input
        .melds
        .iter()
        .filter_map(|meld| match meld.kind() {
            MahjongMeldKind::Kong(kind) => Some(kind),
            _ => None,
        })
        .collect();
    add(&mut values, Fan::FourKongs, u8::from(kongs.len() == 4));
    add(
        &mut values,
        Fan::SevenShiftedPairs,
        u8::from(matches!(form, Form::SevenPairs { shifted: true })),
    );
    add(
        &mut values,
        Fan::ThirteenOrphans,
        u8::from(matches!(form, Form::ThirteenOrphans)),
    );

    let all_terminals = all_tiles.iter().all(|tile| tile.is_terminal());
    add(&mut values, Fan::AllTerminals, u8::from(all_terminals));
    add(&mut values, Fan::LittleFourWinds, u8::from(little_four));
    add(&mut values, Fan::LittleThreeDragons, u8::from(little_three));
    let all_honors = all_tiles.iter().all(|tile| tile.is_honor());
    add(&mut values, Fan::AllHonors, u8::from(all_honors));

    let concealed_pungs =
        concealed_pung_count(input, sets_pair.map(|value| value.0).unwrap_or(&[]));
    add(
        &mut values,
        Fan::FourConcealedPungs,
        u8::from(concealed_pungs == 4),
    );
    let pure_terminal_chows =
        sets_pair.is_some_and(|(sets, pair)| is_pure_terminal_chows(sets, pair));
    add(
        &mut values,
        Fan::PureTerminalChows,
        u8::from(pure_terminal_chows),
    );

    let quadruple_chow = same_chow_count(&chows, 4);
    let four_shifted_pungs = pure_shifted_pung_count(&pungs, 4);
    add(&mut values, Fan::QuadrupleChow, u8::from(quadruple_chow));
    add(
        &mut values,
        Fan::FourPureShiftedPungs,
        u8::from(four_shifted_pungs),
    );
    let four_shifted_chows = pure_shifted_chow_count(&chows, 4);
    add(
        &mut values,
        Fan::FourPureShiftedChows,
        u8::from(four_shifted_chows),
    );
    add(&mut values, Fan::ThreeKongs, u8::from(kongs.len() == 3));

    let terminals_honors = all_tiles.iter().all(|tile| tile.is_terminal_or_honor())
        && all_tiles.iter().any(|tile| tile.is_terminal())
        && all_tiles.iter().any(|tile| tile.is_honor());
    add(
        &mut values,
        Fan::AllTerminalsAndHonors,
        u8::from(terminals_honors),
    );
    add(
        &mut values,
        Fan::SevenPairs,
        u8::from(matches!(form, Form::SevenPairs { shifted: false })),
    );
    add(
        &mut values,
        Fan::GreaterHonorsAndKnittedTiles,
        u8::from(matches!(
            form,
            Form::Knitted {
                greater: true,
                pair: None,
                ..
            }
        )),
    );

    let all_even_pungs = sets_pair.is_some_and(|(sets, pair)| {
        is_standard
            && sets.iter().all(|set| {
                matches!(
                    set.kind,
                    SetKind::Pung(MahjongTileKind::Suited {
                        rank: 2 | 4 | 6 | 8,
                        ..
                    })
                )
            })
            && matches!(
                pair,
                MahjongTileKind::Suited {
                    rank: 2 | 4 | 6 | 8,
                    ..
                }
            )
    });
    add(&mut values, Fan::AllEvenPungs, u8::from(all_even_pungs));
    let (suit_count, has_honor) = suit_summary(&all_tiles);
    let full_flush = suit_count == 1 && !has_honor;
    add(&mut values, Fan::FullFlush, u8::from(full_flush));
    let pure_triple = same_chow_count(&chows, 3);
    let pure_shifted_pungs = pure_shifted_pung_count(&pungs, 3);
    add(
        &mut values,
        Fan::PureTripleChow,
        u8::from(pure_triple && !quadruple_chow),
    );
    add(
        &mut values,
        Fan::PureShiftedPungs,
        u8::from(pure_shifted_pungs && !four_shifted_pungs),
    );
    add(
        &mut values,
        Fan::UpperTiles,
        u8::from(all_suited_in(&all_tiles, 7, 9)),
    );
    add(
        &mut values,
        Fan::MiddleTiles,
        u8::from(all_suited_in(&all_tiles, 4, 6)),
    );
    add(
        &mut values,
        Fan::LowerTiles,
        u8::from(all_suited_in(&all_tiles, 1, 3)),
    );

    let pure_straight = has_chow_starts_same_suit(&chows, &[1, 4, 7]);
    add(&mut values, Fan::PureStraight, u8::from(pure_straight));
    let three_suited_terminal =
        sets_pair.is_some_and(|(sets, pair)| is_three_suited_terminal_chows(sets, pair));
    add(
        &mut values,
        Fan::ThreeSuitedTerminalChows,
        u8::from(three_suited_terminal),
    );
    let pure_shifted_chows = pure_shifted_chow_count(&chows, 3);
    add(
        &mut values,
        Fan::PureShiftedChows,
        u8::from(pure_shifted_chows && !four_shifted_chows),
    );
    let all_fives = sets_pair.is_some_and(|(sets, pair)| {
        is_standard
            && matches!(pair, MahjongTileKind::Suited { rank: 5, .. })
            && sets.iter().all(|set| set_contains_rank(set, 5))
    });
    add(&mut values, Fan::AllFives, u8::from(all_fives));
    let triple_pung = (1..=9).any(|rank| {
        MahjongSuit::ALL
            .iter()
            .all(|suit| pungs.contains(&MahjongTileKind::suited(*suit, rank)))
    });
    add(&mut values, Fan::TriplePung, u8::from(triple_pung));
    add(
        &mut values,
        Fan::ThreeConcealedPungs,
        u8::from(concealed_pungs == 3),
    );

    add(
        &mut values,
        Fan::LesserHonorsAndKnittedTiles,
        u8::from(matches!(
            form,
            Form::Knitted {
                greater: false,
                pair: None,
                ..
            }
        )),
    );
    add(
        &mut values,
        Fan::KnittedStraight,
        u8::from(matches!(form, Form::Knitted { straight: true, .. })),
    );
    add(
        &mut values,
        Fan::UpperFour,
        u8::from(all_suited_in(&all_tiles, 6, 9)),
    );
    add(
        &mut values,
        Fan::LowerFour,
        u8::from(all_suited_in(&all_tiles, 1, 4)),
    );
    add(
        &mut values,
        Fan::BigThreeWinds,
        u8::from(wind_pungs == 3 && !big_four && !little_four),
    );

    let mixed_straight = has_mixed_straight(&chows);
    add(&mut values, Fan::MixedStraight, u8::from(mixed_straight));
    let reversible = all_tiles.iter().all(is_reversible);
    add(&mut values, Fan::ReversibleTiles, u8::from(reversible));
    let mixed_triple = (1..=7).any(|start| {
        MahjongSuit::ALL
            .iter()
            .all(|suit| chows.contains(&(*suit, start)))
    });
    add(&mut values, Fan::MixedTripleChow, u8::from(mixed_triple));
    let mixed_shifted_pungs = has_mixed_shifted_pungs(&pungs);
    add(
        &mut values,
        Fan::MixedShiftedPungs,
        u8::from(mixed_shifted_pungs),
    );

    if input.context.last_wall_tile {
        if input.context.source.is_self_draw() {
            add(&mut values, Fan::LastTileDraw, 1);
        } else if matches!(input.context.source, WinSource::Discard(_)) {
            add(&mut values, Fan::LastTileClaim, 1);
        }
    }
    match input.context.source {
        WinSource::KongReplacement => add(&mut values, Fan::OutWithReplacementTile, 1),
        WinSource::RobbingKong(_) => add(&mut values, Fan::RobbingTheKong, 1),
        _ => {}
    }
    add_kong_fans(&mut values, &kongs);

    let all_pungs = sets_pair.is_some_and(|(sets, _)| {
        is_standard && sets.iter().all(|set| matches!(set.kind, SetKind::Pung(_)))
    });
    add(&mut values, Fan::AllPungs, u8::from(all_pungs));
    add(
        &mut values,
        Fan::HalfFlush,
        u8::from(suit_count == 1 && has_honor),
    );
    add(
        &mut values,
        Fan::MixedShiftedChows,
        u8::from(has_mixed_shifted_chows(&chows)),
    );
    let all_types = MahjongSuit::ALL.iter().all(|suit| {
        all_tiles.iter().any(
            |tile| matches!(tile, MahjongTileKind::Suited { suit: tile_suit, .. } if tile_suit == suit),
        )
    }) && all_tiles
        .iter()
        .any(|tile| matches!(tile, MahjongTileKind::Wind(_)))
        && all_tiles
            .iter()
            .any(|tile| matches!(tile, MahjongTileKind::Dragon(_)));
    add(&mut values, Fan::AllTypes, u8::from(all_types));
    let melded_hand = sets_pair.is_some_and(|(sets, _)| {
        sets.len() == 4
            && sets.iter().all(|set| set.open)
            && matches!(input.context.source, WinSource::Discard(_))
    });
    add(&mut values, Fan::MeldedHand, u8::from(melded_hand));
    add(
        &mut values,
        Fan::TwoDragonPungs,
        u8::from(dragon_pungs == 2 && !big_three_dragons && !little_three),
    );

    let outside = sets_pair.is_some_and(|(sets, pair)| {
        is_standard && pair.is_terminal_or_honor() && sets.iter().all(set_has_terminal_or_honor)
    });
    add(&mut values, Fan::OutsideHand, u8::from(outside));
    let fully_concealed =
        input.melds.iter().all(|meld| !meld.is_open()) && input.context.source.is_self_draw();
    add(
        &mut values,
        Fan::FullyConcealedHand,
        u8::from(fully_concealed),
    );
    if input.context.last_of_kind && !matches!(input.context.source, WinSource::RobbingKong(_)) {
        add(&mut values, Fan::LastTile, 1);
    }

    if !big_three_dragons && !little_three && dragon_pungs == 1 {
        add(&mut values, Fan::DragonPung, 1);
    }
    if !big_four {
        if pungs.contains(&MahjongTileKind::Wind(input.context.prevalent_wind)) {
            add(&mut values, Fan::PrevalentWind, 1);
        }
        if pungs.contains(&MahjongTileKind::Wind(input.context.seat_wind)) {
            add(&mut values, Fan::SeatWind, 1);
        }
    }
    let concealed_hand = input.melds.iter().all(|meld| !meld.is_open())
        && matches!(
            input.context.source,
            WinSource::Discard(_) | WinSource::RobbingKong(_)
        );
    add(&mut values, Fan::ConcealedHand, u8::from(concealed_hand));
    let all_chows = sets_pair.is_some_and(|(sets, pair)| {
        !pair.is_honor()
            && sets
                .iter()
                .all(|set| matches!(set.kind, SetKind::Chow { .. }))
            && (is_standard || matches!(form, Form::Knitted { straight: true, .. }))
    });
    add(&mut values, Fan::AllChows, u8::from(all_chows));

    let kong_tiles: Vec<_> = input
        .melds
        .iter()
        .filter(|meld| matches!(meld.kind(), MahjongMeldKind::Kong(_)))
        .map(|meld| meld.tile())
        .collect();
    let tile_hogs = counts
        .iter()
        .enumerate()
        .filter(|(index, count)| {
            **count == 4
                && !kong_tiles
                    .contains(&MahjongTileKind::from_index34(*index).expect("valid index"))
        })
        .count() as u8;
    if !quadruple_chow {
        add(&mut values, Fan::TileHog, tile_hogs);
    }
    let double_pungs = max_matching(&pungs, |left, right| match (left, right) {
        (
            MahjongTileKind::Suited {
                suit: left_suit,
                rank: left_rank,
            },
            MahjongTileKind::Suited {
                suit: right_suit,
                rank: right_rank,
            },
        ) => left_suit != right_suit && left_rank == right_rank,
        _ => false,
    });
    if !triple_pung {
        add(&mut values, Fan::DoublePung, double_pungs);
    }
    if concealed_pungs == 2 {
        add(&mut values, Fan::TwoConcealedPungs, 1);
    }
    let all_simples = all_tiles.iter().all(|tile| !tile.is_terminal_or_honor());
    add(&mut values, Fan::AllSimples, u8::from(all_simples));

    add_chow_relation_fans(
        &mut values,
        &chows,
        quadruple_chow,
        pure_triple,
        mixed_triple,
        pure_straight,
        four_shifted_chows,
        pure_terminal_chows,
        three_suited_terminal,
    );

    if !big_four && !all_honors && !all_terminals && !terminals_honors {
        let mut terminal_honor_pungs = 0;
        for tile in &pungs {
            match tile {
                MahjongTileKind::Suited { rank: 1 | 9, .. } => terminal_honor_pungs += 1,
                MahjongTileKind::Wind(wind)
                    if *wind != input.context.prevalent_wind
                        && *wind != input.context.seat_wind =>
                {
                    terminal_honor_pungs += 1;
                }
                _ => {}
            }
        }
        add(
            &mut values,
            Fan::PungOfTerminalsOrHonors,
            terminal_honor_pungs,
        );
    }
    if !full_flush && !pure_terminal_chows {
        add(&mut values, Fan::OneVoidedSuit, u8::from(suit_count <= 2));
    }
    if !full_flush
        && !all_chows
        && !values.contains_key(&Fan::UpperTiles)
        && !values.contains_key(&Fan::LowerTiles)
        && !values.contains_key(&Fan::UpperFour)
        && !values.contains_key(&Fan::LowerFour)
        && !three_suited_terminal
    {
        add(&mut values, Fan::NoHonors, u8::from(!has_honor));
    }

    if unique_wait && let Some((sets, pair)) = sets_pair {
        let single = pair == input.winning_tile && !melded_hand && kongs.len() < 4;
        let mut edge = false;
        let mut closed = false;
        for set in sets {
            if let SetKind::Chow { suit, start } = set.kind
                && let MahjongTileKind::Suited {
                    suit: win_suit,
                    rank,
                } = input.winning_tile
                && suit == win_suit
            {
                edge |= (start == 1 && rank == 3) || (start == 7 && rank == 7);
                closed |= rank == start + 1;
            }
        }
        if single {
            add(&mut values, Fan::SingleWait, 1);
        } else if edge {
            add(&mut values, Fan::EdgeWait, 1);
        } else if closed {
            add(&mut values, Fan::ClosedWait, 1);
        }
    }
    if matches!(
        input.context.source,
        WinSource::SelfDraw | WinSource::FlowerReplacement
    ) {
        add(&mut values, Fan::SelfDrawn, 1);
    }

    suppress_implied(
        &mut values,
        form,
        big_four,
        big_three_dragons,
        little_four,
        little_three,
        all_honors,
        all_terminals,
        terminals_honors,
        all_even_pungs,
        fully_concealed,
    );

    if values.is_empty() {
        add(&mut values, Fan::ChickenHand, 1);
    }
    let flower_points = input.context.flower_count;
    if flower_points > 0 {
        values.insert(Fan::FlowerTiles, (flower_points, u16::from(flower_points)));
    }
    let fans: Vec<_> = values
        .into_iter()
        .map(|(fan, (count, points))| FanValue { fan, count, points })
        .collect();
    let points_without_flowers = fans
        .iter()
        .filter(|value| value.fan != Fan::FlowerTiles)
        .map(|value| value.points)
        .sum();
    MahjongScoreResult {
        total_points: points_without_flowers + u16::from(flower_points),
        fans,
        points_without_flowers,
        flower_points,
    }
}
