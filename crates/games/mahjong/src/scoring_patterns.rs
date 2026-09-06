use super::*;

pub(super) fn all_tile_kinds(input: &ScoreInput) -> Vec<MahjongTileKind> {
    let mut tiles = input.concealed.clone();
    for meld in &input.melds {
        tiles.extend(meld.tile_kinds());
    }
    tiles
}

pub(super) fn kind_counts(tiles: &[MahjongTileKind]) -> [u8; 34] {
    let mut counts = [0; 34];
    for tile in tiles {
        counts[tile.index34().expect("scoring tiles exclude flowers")] += 1;
    }
    counts
}

pub(super) fn suit_summary(tiles: &[MahjongTileKind]) -> (usize, bool) {
    let suits = MahjongSuit::ALL
        .iter()
        .filter(|suit| {
            tiles.iter().any(|tile| {
                matches!(tile, MahjongTileKind::Suited { suit: tile_suit, .. } if tile_suit == *suit)
            })
        })
        .count();
    (suits, tiles.iter().any(|tile| tile.is_honor()))
}

pub(super) fn all_suited_in(tiles: &[MahjongTileKind], minimum: u8, maximum: u8) -> bool {
    tiles.iter().all(|tile| {
        matches!(tile, MahjongTileKind::Suited { rank, .. } if *rank >= minimum && *rank <= maximum)
    })
}

pub(super) fn is_nine_gates(input: &ScoreInput) -> bool {
    if !input.melds.is_empty() || !matches!(input.winning_tile, MahjongTileKind::Suited { .. }) {
        return false;
    }
    let mut before = input.concealed.clone();
    let Some(position) = before.iter().position(|tile| *tile == input.winning_tile) else {
        return false;
    };
    before.remove(position);
    let MahjongTileKind::Suited { suit, .. } = input.winning_tile else {
        return false;
    };
    let mut ranks = [0_u8; 9];
    for tile in before {
        let MahjongTileKind::Suited {
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

pub(super) fn concealed_pung_count(input: &ScoreInput, sets: &[Set]) -> usize {
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
                MahjongTileKind::Suited { suit: win_suit, rank }
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

pub(super) fn is_pure_terminal_chows(sets: &[Set], pair: MahjongTileKind) -> bool {
    let MahjongTileKind::Suited { suit, rank: 5 } = pair else {
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

pub(super) fn is_three_suited_terminal_chows(sets: &[Set], pair: MahjongTileKind) -> bool {
    let MahjongTileKind::Suited {
        suit: pair_suit,
        rank: 5,
    } = pair
    else {
        return false;
    };
    let mut used_suits = Vec::new();
    for suit in MahjongSuit::ALL {
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

pub(super) fn same_chow_count(chows: &[(MahjongSuit, u8)], count: usize) -> bool {
    chows
        .iter()
        .any(|chow| chows.iter().filter(|other| *other == chow).count() >= count)
}

pub(super) fn pure_shifted_pung_count(pungs: &[MahjongTileKind], count: usize) -> bool {
    MahjongSuit::ALL.iter().any(|suit| {
        (1..=10 - count as u8).any(|start| {
            (0..count)
                .all(|offset| pungs.contains(&MahjongTileKind::suited(*suit, start + offset as u8)))
        })
    })
}

pub(super) fn pure_shifted_chow_count(chows: &[(MahjongSuit, u8)], count: usize) -> bool {
    MahjongSuit::ALL.iter().any(|suit| {
        [1_u8, 2].iter().any(|step| {
            (1..=7).any(|start| {
                start + step * (count as u8 - 1) <= 7
                    && (0..count)
                        .all(|offset| chows.contains(&(*suit, start + step * offset as u8)))
            })
        })
    })
}

pub(super) fn has_chow_starts_same_suit(chows: &[(MahjongSuit, u8)], starts: &[u8]) -> bool {
    MahjongSuit::ALL
        .iter()
        .any(|suit| starts.iter().all(|start| chows.contains(&(*suit, *start))))
}

pub(super) fn has_mixed_straight(chows: &[(MahjongSuit, u8)]) -> bool {
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
            .all(|(index, start)| chows.contains(&(MahjongSuit::ALL[permutation[index]], *start)))
    })
}

pub(super) fn has_mixed_shifted_chows(chows: &[(MahjongSuit, u8)]) -> bool {
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
                chows.contains(&(MahjongSuit::ALL[permutation[offset]], start + offset as u8))
            })
        })
    })
}

pub(super) fn has_mixed_shifted_pungs(pungs: &[MahjongTileKind]) -> bool {
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
                pungs.contains(&MahjongTileKind::suited(
                    MahjongSuit::ALL[permutation[offset]],
                    start + offset as u8,
                ))
            })
        })
    })
}

pub(super) fn set_contains_rank(set: &Set, rank: u8) -> bool {
    match set.kind {
        SetKind::Chow { start, .. } => rank >= start && rank <= start + 2,
        SetKind::Pung(MahjongTileKind::Suited {
            rank: pung_rank, ..
        }) => rank == pung_rank,
        SetKind::Pung(_) => false,
    }
}

pub(super) fn set_has_terminal_or_honor(set: &Set) -> bool {
    match set.kind {
        SetKind::Chow { start: 1 | 7, .. } => true,
        SetKind::Pung(tile) => tile.is_terminal_or_honor(),
        _ => false,
    }
}

pub(super) fn is_reversible(tile: &MahjongTileKind) -> bool {
    matches!(
        tile,
        MahjongTileKind::Suited {
            suit: MahjongSuit::Dots,
            rank: 1 | 2 | 3 | 4 | 5 | 8 | 9
        } | MahjongTileKind::Suited {
            suit: MahjongSuit::Bamboo,
            rank: 2 | 4 | 5 | 6 | 8 | 9
        } | MahjongTileKind::Dragon(MahjongDragon::White)
    )
}

pub(super) fn max_matching<T>(items: &[T], predicate: impl Fn(&T, &T) -> bool + Copy) -> u8 {
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
pub(super) fn add_chow_relation_fans(
    values: &mut BTreeMap<Fan, (u8, u16)>,
    chows: &[(MahjongSuit, u8)],
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

pub(super) fn add_kong_fans(values: &mut BTreeMap<Fan, (u8, u16)>, kongs: &[MahjongKongKind]) {
    let concealed = kongs
        .iter()
        .filter(|kind| matches!(kind, MahjongKongKind::Concealed))
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
pub(super) fn suppress_implied(
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
