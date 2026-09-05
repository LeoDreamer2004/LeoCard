use std::collections::BTreeMap;
use std::fmt;

use crate::{Dragon, KongKind, Meld, MeldKind, PlayerId, Suit, TileKind, Wind};

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
    Discard(PlayerId),
    KongReplacement,
    FlowerReplacement,
    RobbingKong(PlayerId),
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
    pub seat_wind: Wind,
    pub prevalent_wind: Wind,
    pub last_wall_tile: bool,
    pub last_of_kind: bool,
    pub flower_count: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreInput {
    /// 包含和牌张的立牌，不包含副露与花牌。
    pub concealed: Vec<TileKind>,
    pub melds: Vec<Meld>,
    pub winning_tile: TileKind,
    pub context: WinContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreResult {
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
    TooManyCopies(TileKind),
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
    Chow { suit: Suit, start: u8 },
    Pung(TileKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Set {
    kind: SetKind,
    open: bool,
    kong: Option<KongKind>,
}

#[derive(Clone, Debug)]
enum Form {
    Standard {
        sets: Vec<Set>,
        pair: TileKind,
    },
    SevenPairs {
        shifted: bool,
    },
    ThirteenOrphans,
    Knitted {
        greater: bool,
        straight: bool,
        sets: Vec<Set>,
        pair: Option<TileKind>,
    },
}

pub fn is_complete_hand(concealed: &[TileKind], melds: &[Meld]) -> bool {
    validated_forms(concealed, melds, None).is_ok_and(|forms| !forms.is_empty())
}

pub fn score_hand(input: &ScoreInput) -> Result<ScoreResult, ScoreError> {
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

fn validated_forms(
    concealed: &[TileKind],
    melds: &[Meld],
    winning_tile: Option<TileKind>,
) -> Result<Vec<Form>, ScoreError> {
    if concealed.iter().any(|tile| tile.is_flower()) {
        return Err(ScoreError::FlowerInHand);
    }
    if melds.len() > 4 || concealed.len() + melds.len() * 3 != 14 {
        return Err(ScoreError::InvalidTileCount);
    }
    if winning_tile.is_some_and(|tile| !concealed.contains(&tile)) {
        return Err(ScoreError::WinningTileMissing);
    }
    let mut physical = [0_u8; 34];
    for tile in concealed {
        physical[tile.index34().expect("flowers rejected")] += 1;
    }
    for meld in melds {
        for tile in meld.tile_kinds() {
            let count = &mut physical[tile.index34().expect("melds cannot contain flowers")];
            *count += 1;
        }
    }
    if let Some((index, _)) = physical.iter().enumerate().find(|(_, count)| **count > 4) {
        return Err(ScoreError::TooManyCopies(
            TileKind::from_index34(index).expect("index is in 0..34"),
        ));
    }

    let mut counts = [0_u8; 34];
    for tile in concealed {
        counts[tile.index34().expect("flowers rejected")] += 1;
    }
    let mut forms = Vec::new();
    let mut exposed_sets: Vec<Set> = melds.iter().copied().map(set_from_meld).collect();
    for pair_index in 0..34 {
        if counts[pair_index] < 2 {
            continue;
        }
        counts[pair_index] -= 2;
        let needed = 4 - melds.len();
        let mut concealed_sets = Vec::new();
        decompose_sets(&mut counts, needed, &mut concealed_sets, &mut |sets| {
            let mut all = exposed_sets.clone();
            all.extend_from_slice(sets);
            forms.push(Form::Standard {
                sets: all,
                pair: TileKind::from_index34(pair_index).expect("pair index is valid"),
            });
        });
        counts[pair_index] += 2;
    }
    if melds.is_empty() {
        if is_seven_pairs(&counts) {
            forms.push(Form::SevenPairs {
                shifted: is_seven_shifted_pairs(&counts),
            });
        }
        if is_thirteen_orphans(&counts) {
            forms.push(Form::ThirteenOrphans);
        }
        for greater in knitted_variants(&counts) {
            forms.push(Form::Knitted {
                greater,
                straight: has_knitted_straight(&counts),
                sets: Vec::new(),
                pair: None,
            });
        }
    }
    if melds.len() <= 1 {
        for (mut sets, pair) in knitted_straight_remainders(&counts, melds.len()) {
            let mut all = exposed_sets.clone();
            all.append(&mut sets);
            forms.push(Form::Knitted {
                greater: false,
                straight: true,
                sets: all,
                pair: Some(pair),
            });
        }
    }
    exposed_sets.clear();
    Ok(forms)
}

fn set_from_meld(meld: Meld) -> Set {
    let kind = match (meld.kind(), meld.tile()) {
        (MeldKind::Chow, TileKind::Suited { suit, rank }) => SetKind::Chow { suit, start: rank },
        (MeldKind::Pung | MeldKind::Kong(_), tile) => SetKind::Pung(tile),
        (MeldKind::Chow, _) => unreachable!("validated Meld::chow is suited"),
    };
    Set {
        kind,
        open: meld.is_open(),
        kong: match meld.kind() {
            MeldKind::Kong(kind) => Some(kind),
            MeldKind::Chow | MeldKind::Pung => None,
        },
    }
}

fn decompose_sets(
    counts: &mut [u8; 34],
    needed: usize,
    current: &mut Vec<Set>,
    emit: &mut impl FnMut(&[Set]),
) {
    if current.len() == needed {
        if counts.iter().all(|count| *count == 0) {
            emit(current);
        }
        return;
    }
    let Some(index) = counts.iter().position(|count| *count > 0) else {
        return;
    };
    if counts[index] >= 3 {
        counts[index] -= 3;
        current.push(Set {
            kind: SetKind::Pung(TileKind::from_index34(index).expect("valid tile index")),
            open: false,
            kong: None,
        });
        decompose_sets(counts, needed, current, emit);
        current.pop();
        counts[index] += 3;
    }
    if index < 27 && index % 9 <= 6 && counts[index + 1] > 0 && counts[index + 2] > 0 {
        counts[index] -= 1;
        counts[index + 1] -= 1;
        counts[index + 2] -= 1;
        current.push(Set {
            kind: SetKind::Chow {
                suit: Suit::ALL[index / 9],
                start: (index % 9 + 1) as u8,
            },
            open: false,
            kong: None,
        });
        decompose_sets(counts, needed, current, emit);
        current.pop();
        counts[index] += 1;
        counts[index + 1] += 1;
        counts[index + 2] += 1;
    }
}

fn is_seven_pairs(counts: &[u8; 34]) -> bool {
    counts.iter().all(|count| count % 2 == 0)
        && counts.iter().map(|count| count / 2).sum::<u8>() == 7
}

fn is_seven_shifted_pairs(counts: &[u8; 34]) -> bool {
    Suit::ALL.iter().any(|suit| {
        (1..=3).any(|start| {
            (1..=9).all(|rank| {
                let expected = u8::from(rank >= start && rank < start + 7) * 2;
                counts[suit.index() * 9 + rank - 1] == expected
            }) && counts[27..].iter().all(|count| *count == 0)
        })
    })
}

fn is_thirteen_orphans(counts: &[u8; 34]) -> bool {
    const REQUIRED: [usize; 13] = [0, 8, 9, 17, 18, 26, 27, 28, 29, 30, 31, 32, 33];
    REQUIRED.iter().all(|index| counts[*index] >= 1)
        && REQUIRED.iter().map(|index| counts[*index]).sum::<u8>() == 14
        && counts
            .iter()
            .enumerate()
            .all(|(index, count)| REQUIRED.contains(&index) || *count == 0)
}

fn knitted_patterns() -> [[usize; 9]; 6] {
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    permutations.map(|permutation| {
        let mut indices = [0; 9];
        for (line, ranks) in [[1, 4, 7], [2, 5, 8], [3, 6, 9]].iter().enumerate() {
            let suit = permutation[line];
            for (offset, rank) in ranks.iter().enumerate() {
                indices[line * 3 + offset] = suit * 9 + rank - 1;
            }
        }
        indices
    })
}

fn knitted_variants(counts: &[u8; 34]) -> Vec<bool> {
    if counts.iter().any(|count| *count > 1) || counts.iter().sum::<u8>() != 14 {
        return Vec::new();
    }
    knitted_patterns()
        .iter()
        .filter_map(|pattern| {
            let allowed = counts
                .iter()
                .enumerate()
                .all(|(index, count)| *count == 0 || index >= 27 || pattern.contains(&index));
            if !allowed {
                return None;
            }
            let honors = counts[27..].iter().sum::<u8>();
            Some(honors == 7)
        })
        .collect()
}

fn has_knitted_straight(counts: &[u8; 34]) -> bool {
    knitted_patterns()
        .iter()
        .any(|pattern| pattern.iter().all(|index| counts[*index] == 1))
}

fn knitted_straight_remainders(counts: &[u8; 34], meld_count: usize) -> Vec<(Vec<Set>, TileKind)> {
    let mut results = Vec::new();
    for pattern in knitted_patterns() {
        if !pattern.iter().all(|index| counts[*index] >= 1) {
            continue;
        }
        let mut rest = *counts;
        for index in &pattern {
            rest[*index] -= 1;
        }
        match meld_count {
            1 => {
                if rest.iter().filter(|count| **count == 2).count() == 1
                    && rest.iter().sum::<u8>() == 2
                {
                    let pair = rest.iter().position(|count| *count == 2).expect("one pair");
                    results.push((
                        Vec::new(),
                        TileKind::from_index34(pair).expect("valid pair index"),
                    ));
                }
            }
            0 => {
                for pair in 0..34 {
                    if rest[pair] < 2 {
                        continue;
                    }
                    let mut candidate = rest;
                    candidate[pair] -= 2;
                    decompose_sets(&mut candidate, 1, &mut Vec::new(), &mut |sets| {
                        results.push((
                            sets.to_vec(),
                            TileKind::from_index34(pair).expect("valid pair index"),
                        ));
                    });
                }
            }
            _ => {}
        }
    }
    results
}

fn unique_wait(concealed: &[TileKind], melds: &[Meld], winning: TileKind) -> bool {
    let mut before = concealed.to_vec();
    let Some(position) = before.iter().position(|tile| *tile == winning) else {
        return false;
    };
    before.remove(position);
    let mut wins = 0;
    for index in 0..34 {
        let tile = TileKind::from_index34(index).expect("valid index");
        let used = before.iter().filter(|held| **held == tile).count()
            + melds
                .iter()
                .flat_map(|meld| meld.tile_kinds())
                .filter(|held| *held == tile)
                .count();
        if used < 4 {
            before.push(tile);
            if is_complete_hand(&before, melds) {
                wins += 1;
            }
            before.pop();
        }
    }
    wins == 1
}

fn score_form(input: &ScoreInput, form: &Form, unique_wait: bool) -> ScoreResult {
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
    let pungs: Vec<TileKind> = sets_pair
        .map(|(sets, _)| {
            sets.iter()
                .filter_map(|set| match set.kind {
                    SetKind::Pung(tile) => Some(tile),
                    SetKind::Chow { .. } => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let chows: Vec<(Suit, u8)> = sets_pair
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
        .filter(|tile| matches!(tile, TileKind::Wind(_)))
        .count();
    let dragon_pungs = pungs
        .iter()
        .filter(|tile| matches!(tile, TileKind::Dragon(_)))
        .count();
    let big_four = wind_pungs == 4;
    let big_three_dragons = dragon_pungs == 3;
    let little_four =
        sets_pair.is_some_and(|(_, pair)| wind_pungs == 3 && matches!(pair, TileKind::Wind(_)));
    let little_three =
        sets_pair.is_some_and(|(_, pair)| dragon_pungs == 2 && matches!(pair, TileKind::Dragon(_)));
    add(&mut values, Fan::BigFourWinds, u8::from(big_four));
    add(
        &mut values,
        Fan::BigThreeDragons,
        u8::from(big_three_dragons),
    );

    let all_green = all_tiles.iter().all(|tile| {
        matches!(
            tile,
            TileKind::Suited {
                suit: Suit::Bamboo,
                rank: 2 | 3 | 4 | 6 | 8
            } | TileKind::Dragon(Dragon::Green)
        )
    });
    add(&mut values, Fan::AllGreen, u8::from(all_green));
    add(&mut values, Fan::NineGates, u8::from(is_nine_gates(input)));

    let kongs: Vec<KongKind> = input
        .melds
        .iter()
        .filter_map(|meld| match meld.kind() {
            MeldKind::Kong(kind) => Some(kind),
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
                    SetKind::Pung(TileKind::Suited {
                        rank: 2 | 4 | 6 | 8,
                        ..
                    })
                )
            })
            && matches!(
                pair,
                TileKind::Suited {
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
            && matches!(pair, TileKind::Suited { rank: 5, .. })
            && sets.iter().all(|set| set_contains_rank(set, 5))
    });
    add(&mut values, Fan::AllFives, u8::from(all_fives));
    let triple_pung = (1..=9).any(|rank| {
        Suit::ALL
            .iter()
            .all(|suit| pungs.contains(&TileKind::suited(*suit, rank)))
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
    let mixed_triple =
        (1..=7).any(|start| Suit::ALL.iter().all(|suit| chows.contains(&(*suit, start))));
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
    let all_types = Suit::ALL.iter().all(|suit| {
        all_tiles.iter().any(
            |tile| matches!(tile, TileKind::Suited { suit: tile_suit, .. } if tile_suit == suit),
        )
    }) && all_tiles
        .iter()
        .any(|tile| matches!(tile, TileKind::Wind(_)))
        && all_tiles
            .iter()
            .any(|tile| matches!(tile, TileKind::Dragon(_)));
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
        if pungs.contains(&TileKind::Wind(input.context.prevalent_wind)) {
            add(&mut values, Fan::PrevalentWind, 1);
        }
        if pungs.contains(&TileKind::Wind(input.context.seat_wind)) {
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
        .filter(|meld| matches!(meld.kind(), MeldKind::Kong(_)))
        .map(|meld| meld.tile())
        .collect();
    let tile_hogs = counts
        .iter()
        .enumerate()
        .filter(|(index, count)| {
            **count == 4
                && !kong_tiles.contains(&TileKind::from_index34(*index).expect("valid index"))
        })
        .count() as u8;
    if !quadruple_chow {
        add(&mut values, Fan::TileHog, tile_hogs);
    }
    let double_pungs = max_matching(&pungs, |left, right| match (left, right) {
        (
            TileKind::Suited {
                suit: left_suit,
                rank: left_rank,
            },
            TileKind::Suited {
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
                TileKind::Suited { rank: 1 | 9, .. } => terminal_honor_pungs += 1,
                TileKind::Wind(wind)
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

    if unique_wait {
        if let Some((sets, pair)) = sets_pair {
            let single = pair == input.winning_tile && !melded_hand && kongs.len() < 4;
            let mut edge = false;
            let mut closed = false;
            for set in sets {
                if let SetKind::Chow { suit, start } = set.kind
                    && let TileKind::Suited {
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
    ScoreResult {
        total_points: points_without_flowers + u16::from(flower_points),
        fans,
        points_without_flowers,
        flower_points,
    }
}

fn all_tile_kinds(input: &ScoreInput) -> Vec<TileKind> {
    let mut tiles = input.concealed.clone();
    for meld in &input.melds {
        tiles.extend(meld.tile_kinds());
    }
    tiles
}

fn kind_counts(tiles: &[TileKind]) -> [u8; 34] {
    let mut counts = [0; 34];
    for tile in tiles {
        counts[tile.index34().expect("scoring tiles exclude flowers")] += 1;
    }
    counts
}

fn suit_summary(tiles: &[TileKind]) -> (usize, bool) {
    let suits = Suit::ALL
        .iter()
        .filter(|suit| {
            tiles.iter().any(|tile| {
                matches!(tile, TileKind::Suited { suit: tile_suit, .. } if tile_suit == *suit)
            })
        })
        .count();
    (suits, tiles.iter().any(|tile| tile.is_honor()))
}

fn all_suited_in(tiles: &[TileKind], minimum: u8, maximum: u8) -> bool {
    tiles.iter().all(|tile| {
        matches!(tile, TileKind::Suited { rank, .. } if *rank >= minimum && *rank <= maximum)
    })
}

fn is_nine_gates(input: &ScoreInput) -> bool {
    if !input.melds.is_empty() || !matches!(input.winning_tile, TileKind::Suited { .. }) {
        return false;
    }
    let mut before = input.concealed.clone();
    let Some(position) = before.iter().position(|tile| *tile == input.winning_tile) else {
        return false;
    };
    before.remove(position);
    let TileKind::Suited { suit, .. } = input.winning_tile else {
        return false;
    };
    let mut ranks = [0_u8; 9];
    for tile in before {
        let TileKind::Suited {
            suit: tile_suit,
            rank,
        } = tile
        else {
            return false;
        };
        if tile_suit != suit {
            return false;
        }
        ranks[rank as usize - 1] += 1;
    }
    ranks == [3, 1, 1, 1, 1, 1, 1, 1, 3]
}

fn concealed_pung_count(input: &ScoreInput, sets: &[Set]) -> usize {
    let mut count = sets
        .iter()
        .filter(|set| matches!(set.kind, SetKind::Pung(_)) && !set.open)
        .count();
    if !input.context.source.is_self_draw() {
        let winning_pung = sets.iter().any(|set| {
            matches!(set.kind, SetKind::Pung(tile) if tile == input.winning_tile) && !set.open
        });
        let winning_can_belong_elsewhere = sets.iter().any(|set| match set.kind {
            SetKind::Chow { suit, start } => matches!(
                input.winning_tile,
                TileKind::Suited { suit: win_suit, rank }
                    if suit == win_suit && rank >= start && rank <= start + 2
            ),
            SetKind::Pung(_) => false,
        });
        if winning_pung && !winning_can_belong_elsewhere {
            count -= 1;
        }
    }
    count
}

fn is_pure_terminal_chows(sets: &[Set], pair: TileKind) -> bool {
    let TileKind::Suited { suit, rank: 5 } = pair else {
        return false;
    };
    let starts: Vec<_> = sets
        .iter()
        .filter_map(|set| match set.kind {
            SetKind::Chow {
                suit: chow_suit,
                start,
            } if chow_suit == suit => Some(start),
            _ => None,
        })
        .collect();
    starts.len() == 4
        && starts.iter().filter(|start| **start == 1).count() == 2
        && starts.iter().filter(|start| **start == 7).count() == 2
}

fn is_three_suited_terminal_chows(sets: &[Set], pair: TileKind) -> bool {
    let TileKind::Suited {
        suit: pair_suit,
        rank: 5,
    } = pair
    else {
        return false;
    };
    let mut used_suits = Vec::new();
    for suit in Suit::ALL {
        if suit == pair_suit {
            continue;
        }
        if sets
            .iter()
            .any(|set| matches!(set.kind, SetKind::Chow { suit: s, start: 1 } if s == suit))
            && sets
                .iter()
                .any(|set| matches!(set.kind, SetKind::Chow { suit: s, start: 7 } if s == suit))
        {
            used_suits.push(suit);
        }
    }
    used_suits.len() == 2 && sets.len() == 4
}

fn same_chow_count(chows: &[(Suit, u8)], count: usize) -> bool {
    chows
        .iter()
        .any(|chow| chows.iter().filter(|other| *other == chow).count() >= count)
}

fn pure_shifted_pung_count(pungs: &[TileKind], count: usize) -> bool {
    Suit::ALL.iter().any(|suit| {
        (1..=10 - count as u8).any(|start| {
            (0..count).all(|offset| pungs.contains(&TileKind::suited(*suit, start + offset as u8)))
        })
    })
}

fn pure_shifted_chow_count(chows: &[(Suit, u8)], count: usize) -> bool {
    Suit::ALL.iter().any(|suit| {
        [1_u8, 2].iter().any(|step| {
            (1..=7).any(|start| {
                start + step * (count as u8 - 1) <= 7
                    && (0..count)
                        .all(|offset| chows.contains(&(*suit, start + step * offset as u8)))
            })
        })
    })
}

fn has_chow_starts_same_suit(chows: &[(Suit, u8)], starts: &[u8]) -> bool {
    Suit::ALL
        .iter()
        .any(|suit| starts.iter().all(|start| chows.contains(&(*suit, *start))))
}

fn has_mixed_straight(chows: &[(Suit, u8)]) -> bool {
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    permutations.iter().any(|permutation| {
        [1_u8, 4, 7]
            .iter()
            .enumerate()
            .all(|(index, start)| chows.contains(&(Suit::ALL[permutation[index]], *start)))
    })
}

fn has_mixed_shifted_chows(chows: &[(Suit, u8)]) -> bool {
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    permutations.iter().any(|permutation| {
        (1..=5).any(|start| {
            (0..3).all(|offset| {
                chows.contains(&(Suit::ALL[permutation[offset]], start + offset as u8))
            })
        })
    })
}

fn has_mixed_shifted_pungs(pungs: &[TileKind]) -> bool {
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    permutations.iter().any(|permutation| {
        (1..=7).any(|start| {
            (0..3).all(|offset| {
                pungs.contains(&TileKind::suited(
                    Suit::ALL[permutation[offset]],
                    start + offset as u8,
                ))
            })
        })
    })
}

fn set_contains_rank(set: &Set, rank: u8) -> bool {
    match set.kind {
        SetKind::Chow { start, .. } => rank >= start && rank <= start + 2,
        SetKind::Pung(TileKind::Suited {
            rank: pung_rank, ..
        }) => rank == pung_rank,
        SetKind::Pung(_) => false,
    }
}

fn set_has_terminal_or_honor(set: &Set) -> bool {
    match set.kind {
        SetKind::Chow { start: 1 | 7, .. } => true,
        SetKind::Pung(tile) => tile.is_terminal_or_honor(),
        _ => false,
    }
}

fn is_reversible(tile: &TileKind) -> bool {
    matches!(
        tile,
        TileKind::Suited {
            suit: Suit::Dots,
            rank: 1 | 2 | 3 | 4 | 5 | 8 | 9
        } | TileKind::Suited {
            suit: Suit::Bamboo,
            rank: 2 | 4 | 5 | 6 | 8 | 9
        } | TileKind::Dragon(Dragon::White)
    )
}

fn max_matching<T>(items: &[T], predicate: impl Fn(&T, &T) -> bool + Copy) -> u8 {
    fn search<T>(items: &[T], used: u16, predicate: impl Fn(&T, &T) -> bool + Copy) -> u8 {
        let Some(left) = (0..items.len()).find(|index| used & (1 << index) == 0) else {
            return 0;
        };
        let mut best = search(items, used | (1 << left), predicate);
        for right in left + 1..items.len() {
            if used & (1 << right) == 0 && predicate(&items[left], &items[right]) {
                best = best.max(1 + search(items, used | (1 << left) | (1 << right), predicate));
            }
        }
        best
    }
    search(items, 0, predicate)
}

#[allow(clippy::too_many_arguments)]
fn add_chow_relation_fans(
    values: &mut BTreeMap<Fan, (u8, u16)>,
    chows: &[(Suit, u8)],
    quadruple: bool,
    pure_triple: bool,
    mixed_triple: bool,
    pure_straight: bool,
    four_shifted: bool,
    pure_terminal: bool,
    three_suited_terminal: bool,
) {
    if quadruple || pure_terminal {
        return;
    }
    let mut pure_double = max_matching(chows, |left, right| left == right);
    let mut mixed_double =
        max_matching(chows, |left, right| left.0 != right.0 && left.1 == right.1);
    if pure_triple {
        pure_double = 0;
        mixed_double = mixed_double.min(1);
    }
    if mixed_triple {
        if pure_double > 0 {
            pure_double = 1;
            mixed_double = 0;
        } else {
            mixed_double = mixed_double.min(1);
        }
    }
    if three_suited_terminal {
        mixed_double = 0;
    }
    if pure_double > 0 {
        values.insert(Fan::PureDoubleChow, (pure_double, u16::from(pure_double)));
    }
    if mixed_double > 0 {
        values.insert(
            Fan::MixedDoubleChow,
            (mixed_double, u16::from(mixed_double)),
        );
    }
    if !pure_straight && !four_shifted {
        let short = max_matching(chows, |left, right| {
            left.0 == right.0 && left.1.abs_diff(right.1) == 3
        });
        if short > 0 {
            values.insert(Fan::ShortStraight, (short, u16::from(short)));
        }
    }
    if !pure_straight && !four_shifted && !three_suited_terminal {
        let terminals = max_matching(chows, |left, right| {
            left.0 == right.0 && [left.1, right.1].contains(&1) && [left.1, right.1].contains(&7)
        });
        if terminals > 0 {
            values.insert(Fan::TwoTerminalChows, (terminals, u16::from(terminals)));
        }
    }
}

fn add_kong_fans(values: &mut BTreeMap<Fan, (u8, u16)>, kongs: &[KongKind]) {
    let concealed = kongs
        .iter()
        .filter(|kind| matches!(kind, KongKind::Concealed))
        .count();
    let melded = kongs.len() - concealed;
    if kongs.len() >= 3 {
        match concealed {
            1 => {
                values.insert(Fan::ConcealedKong, (1, 2));
            }
            2 => {
                values.insert(Fan::TwoConcealedKongs, (1, 8));
            }
            _ => {}
        }
        return;
    }
    match (concealed, melded) {
        (2, 0) => {
            values.insert(Fan::TwoConcealedKongs, (1, 8));
        }
        (0, 2) => {
            values.insert(Fan::TwoMeldedKongs, (1, 4));
        }
        (1, 1) => {
            // 2014 版规则在“双明杠”条目中明确规定一明一暗合计 6 分。
            values.insert(Fan::TwoMeldedKongs, (1, 6));
        }
        (1, 0) => {
            values.insert(Fan::ConcealedKong, (1, 2));
        }
        (0, 1) => {
            values.insert(Fan::MeldedKong, (1, 1));
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn suppress_implied(
    values: &mut BTreeMap<Fan, (u8, u16)>,
    form: &Form,
    big_four: bool,
    big_three_dragons: bool,
    little_four: bool,
    little_three: bool,
    all_honors: bool,
    all_terminals: bool,
    terminals_honors: bool,
    all_even_pungs: bool,
    fully_concealed: bool,
) {
    let remove = |values: &mut BTreeMap<Fan, (u8, u16)>, fans: &[Fan]| {
        for fan in fans {
            values.remove(fan);
        }
    };
    if big_four {
        remove(
            values,
            &[
                Fan::BigThreeWinds,
                Fan::AllPungs,
                Fan::PrevalentWind,
                Fan::SeatWind,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if big_three_dragons || little_three {
        remove(values, &[Fan::TwoDragonPungs, Fan::DragonPung]);
    }
    if little_four {
        remove(values, &[Fan::BigThreeWinds, Fan::PungOfTerminalsOrHonors]);
    }
    if all_honors || all_terminals || terminals_honors {
        remove(
            values,
            &[
                Fan::AllPungs,
                Fan::OutsideHand,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if all_terminals {
        values.remove(&Fan::NoHonors);
    }
    if all_even_pungs {
        remove(values, &[Fan::AllPungs, Fan::AllSimples]);
    }
    if fully_concealed {
        values.remove(&Fan::SelfDrawn);
    }
    match form {
        Form::SevenPairs { shifted: true } => remove(
            values,
            &[
                Fan::SevenPairs,
                Fan::FullFlush,
                Fan::ConcealedHand,
                Fan::SingleWait,
            ],
        ),
        Form::SevenPairs { shifted: false } => {
            remove(values, &[Fan::ConcealedHand, Fan::SingleWait]);
        }
        Form::ThirteenOrphans => remove(
            values,
            &[
                Fan::AllTerminalsAndHonors,
                Fan::AllTypes,
                Fan::ConcealedHand,
                Fan::SingleWait,
            ],
        ),
        Form::Knitted { pair: None, .. } => {
            remove(values, &[Fan::AllTypes, Fan::ConcealedHand]);
        }
        Form::Knitted { pair: Some(_), .. } => {}
        Form::Standard { .. } => {}
    }
    if values.contains_key(&Fan::NineGates) {
        remove(
            values,
            &[
                Fan::FullFlush,
                Fan::ConcealedHand,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if values.contains_key(&Fan::FourKongs) {
        remove(
            values,
            &[
                Fan::ThreeKongs,
                Fan::TwoMeldedKongs,
                Fan::MeldedKong,
                Fan::SingleWait,
            ],
        );
    }
    if values.contains_key(&Fan::FourConcealedPungs) {
        remove(
            values,
            &[
                Fan::AllPungs,
                Fan::ThreeConcealedPungs,
                Fan::TwoConcealedPungs,
                Fan::ConcealedHand,
            ],
        );
    } else if values.contains_key(&Fan::ThreeConcealedPungs) {
        values.remove(&Fan::TwoConcealedPungs);
    }
    if values.contains_key(&Fan::FullFlush) {
        values.remove(&Fan::NoHonors);
    }
    if values.contains_key(&Fan::HalfFlush) || all_honors {
        values.remove(&Fan::OneVoidedSuit);
    }
    if values.contains_key(&Fan::AllChows) {
        values.remove(&Fan::NoHonors);
    }
    if values.contains_key(&Fan::MeldedHand) {
        values.remove(&Fan::SingleWait);
    }
    if values.contains_key(&Fan::ReversibleTiles) {
        values.remove(&Fan::OneVoidedSuit);
    }
    if values.contains_key(&Fan::OutWithReplacementTile) || values.contains_key(&Fan::LastTileDraw)
    {
        values.remove(&Fan::SelfDrawn);
    }
    if values.contains_key(&Fan::RobbingTheKong) {
        values.remove(&Fan::LastTile);
    }
    if values.contains_key(&Fan::PureTerminalChows) {
        remove(
            values,
            &[
                Fan::SevenPairs,
                Fan::FullFlush,
                Fan::AllChows,
                Fan::NoHonors,
                Fan::PureDoubleChow,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains_key(&Fan::QuadrupleChow) {
        remove(
            values,
            &[
                Fan::PureTripleChow,
                Fan::PureShiftedPungs,
                Fan::TileHog,
                Fan::PureDoubleChow,
            ],
        );
    }
    if values.contains_key(&Fan::FourPureShiftedPungs) {
        remove(
            values,
            &[Fan::PureShiftedPungs, Fan::PureTripleChow, Fan::AllPungs],
        );
    }
    if values.contains_key(&Fan::FourPureShiftedChows) {
        remove(
            values,
            &[
                Fan::PureShiftedChows,
                Fan::ShortStraight,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains_key(&Fan::PureTripleChow) {
        remove(values, &[Fan::PureShiftedPungs, Fan::PureDoubleChow]);
    }
    if values.contains_key(&Fan::ThreeSuitedTerminalChows) {
        remove(
            values,
            &[
                Fan::AllChows,
                Fan::NoHonors,
                Fan::MixedDoubleChow,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains_key(&Fan::AllFives) {
        remove(values, &[Fan::AllSimples, Fan::NoHonors]);
    }
    for fan in [
        Fan::UpperTiles,
        Fan::LowerTiles,
        Fan::UpperFour,
        Fan::LowerFour,
    ] {
        if values.contains_key(&fan) {
            values.remove(&Fan::NoHonors);
        }
    }
    if values.contains_key(&Fan::MiddleTiles) {
        values.remove(&Fan::AllSimples);
    }
    if values.contains_key(&Fan::AllSimples) || all_even_pungs {
        values.remove(&Fan::NoHonors);
    }
    if values.contains_key(&Fan::TwoDragonPungs) {
        values.remove(&Fan::DragonPung);
    }
    if values.contains_key(&Fan::TwoConcealedKongs) {
        values.remove(&Fan::ConcealedKong);
    }
    if values.contains_key(&Fan::TwoMeldedKongs) {
        remove(values, &[Fan::MeldedKong, Fan::ConcealedKong]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(suit: Suit, rank: u8) -> TileKind {
        TileKind::suited(suit, rank)
    }

    fn context(source: WinSource) -> WinContext {
        WinContext {
            source,
            seat_wind: Wind::East,
            prevalent_wind: Wind::East,
            last_wall_tile: false,
            last_of_kind: false,
            flower_count: 0,
        }
    }

    fn fan(result: &ScoreResult, target: Fan) -> Option<FanValue> {
        result
            .fans
            .iter()
            .find(|value| value.fan == target)
            .copied()
    }

    #[test]
    fn recognizes_thirteen_orphans_and_excludes_flowers_from_minimum() {
        let mut concealed = vec![
            c(Suit::Characters, 1),
            c(Suit::Characters, 9),
            c(Suit::Bamboo, 1),
            c(Suit::Bamboo, 9),
            c(Suit::Dots, 1),
            c(Suit::Dots, 9),
            TileKind::Wind(Wind::East),
            TileKind::Wind(Wind::South),
            TileKind::Wind(Wind::West),
            TileKind::Wind(Wind::North),
            TileKind::Dragon(Dragon::Red),
            TileKind::Dragon(Dragon::Green),
            TileKind::Dragon(Dragon::White),
        ];
        concealed.push(TileKind::Wind(Wind::East));
        let result = score_hand(&ScoreInput {
            winning_tile: TileKind::Wind(Wind::East),
            concealed,
            melds: Vec::new(),
            context: WinContext {
                flower_count: 8,
                ..context(WinSource::SelfDraw)
            },
        })
        .unwrap();
        assert!(fan(&result, Fan::ThirteenOrphans).is_some());
        assert_eq!(result.points_without_flowers, 92);
        assert_eq!(result.total_points, 100);
    }

    #[test]
    fn detects_pure_straight_and_does_not_repeat_short_straight() {
        let concealed = vec![
            c(Suit::Characters, 1),
            c(Suit::Characters, 2),
            c(Suit::Characters, 3),
            c(Suit::Characters, 4),
            c(Suit::Characters, 5),
            c(Suit::Characters, 6),
            c(Suit::Characters, 7),
            c(Suit::Characters, 8),
            c(Suit::Characters, 9),
            c(Suit::Dots, 2),
            c(Suit::Dots, 3),
            c(Suit::Dots, 4),
            TileKind::Dragon(Dragon::Red),
            TileKind::Dragon(Dragon::Red),
        ];
        let result = score_hand(&ScoreInput {
            concealed,
            melds: Vec::new(),
            winning_tile: TileKind::Dragon(Dragon::Red),
            context: context(WinSource::Discard(PlayerId(1))),
        })
        .unwrap();
        assert!(fan(&result, Fan::PureStraight).is_some());
        assert!(fan(&result, Fan::ShortStraight).is_none());
        assert!(fan(&result, Fan::TwoTerminalChows).is_none());
    }

    #[test]
    fn mixed_melded_and_concealed_kong_scores_six_in_2014_rules() {
        let concealed = vec![
            c(Suit::Characters, 3),
            c(Suit::Characters, 4),
            c(Suit::Characters, 5),
            c(Suit::Dots, 7),
            c(Suit::Dots, 7),
        ];
        let melds = vec![
            Meld::melded_kong(c(Suit::Bamboo, 2), PlayerId(1)),
            Meld::concealed_kong(c(Suit::Dots, 4)),
            Meld::pung(TileKind::Dragon(Dragon::Red), PlayerId(2)),
        ];
        let result = score_hand(&ScoreInput {
            concealed,
            melds,
            winning_tile: c(Suit::Dots, 7),
            context: context(WinSource::Discard(PlayerId(3))),
        })
        .unwrap();
        assert_eq!(fan(&result, Fan::TwoMeldedKongs).unwrap().points, 6);
        assert!(fan(&result, Fan::MeldedKong).is_none());
        assert!(fan(&result, Fan::ConcealedKong).is_none());
    }

    #[test]
    fn knitted_straight_keeps_the_remaining_set_and_pair_fans() {
        let concealed = vec![
            c(Suit::Characters, 1),
            c(Suit::Characters, 4),
            c(Suit::Characters, 7),
            c(Suit::Bamboo, 2),
            c(Suit::Bamboo, 5),
            c(Suit::Bamboo, 8),
            c(Suit::Dots, 3),
            c(Suit::Dots, 6),
            c(Suit::Dots, 9),
            TileKind::Dragon(Dragon::Red),
            TileKind::Dragon(Dragon::Red),
            TileKind::Dragon(Dragon::Red),
            TileKind::Wind(Wind::East),
            TileKind::Wind(Wind::East),
        ];
        let result = score_hand(&ScoreInput {
            concealed,
            melds: Vec::new(),
            winning_tile: TileKind::Wind(Wind::East),
            context: context(WinSource::Discard(PlayerId(1))),
        })
        .unwrap();
        assert!(fan(&result, Fan::KnittedStraight).is_some());
        assert!(fan(&result, Fan::AllTypes).is_some());
        assert!(fan(&result, Fan::DragonPung).is_some());
        assert!(fan(&result, Fan::ConcealedHand).is_some());
        assert!(fan(&result, Fan::SingleWait).is_some());
    }
}
