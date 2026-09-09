use super::{ScoreInput, Set, SetKind};
use crate::{MahjongDragon, MahjongSuit, MahjongTileKind};

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
