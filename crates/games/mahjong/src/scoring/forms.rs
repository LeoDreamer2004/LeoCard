use super::{Form, ScoreError, Set, SetKind, is_complete_hand};
use crate::{MahjongMeldKind, MahjongSuit, MahjongTileKind, Meld};

pub(super) fn validated_forms(
    concealed: &[MahjongTileKind],
    melds: &[Meld],
    winning_tile: Option<MahjongTileKind>,
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
            MahjongTileKind::from_index34(index).expect("index is in 0..34"),
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
                pair: MahjongTileKind::from_index34(pair_index).expect("pair index is valid"),
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
        (MahjongMeldKind::Chow, MahjongTileKind::Suited { suit, rank }) => {
            SetKind::Chow { suit, start: rank }
        }
        (MahjongMeldKind::Pung | MahjongMeldKind::Kong(_), tile) => SetKind::Pung(tile),
        (MahjongMeldKind::Chow, _) => unreachable!("validated Meld::chow is suited"),
    };
    Set {
        kind,
        open: meld.is_open(),
        kong: match meld.kind() {
            MahjongMeldKind::Kong(kind) => Some(kind),
            MahjongMeldKind::Chow | MahjongMeldKind::Pung => None,
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
            kind: SetKind::Pung(MahjongTileKind::from_index34(index).expect("valid tile index")),
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
                suit: MahjongSuit::ALL[index / 9],
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
    MahjongSuit::ALL.iter().any(|suit| {
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

fn knitted_straight_remainders(
    counts: &[u8; 34],
    meld_count: usize,
) -> Vec<(Vec<Set>, MahjongTileKind)> {
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
                        MahjongTileKind::from_index34(pair).expect("valid pair index"),
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
                            MahjongTileKind::from_index34(pair).expect("valid pair index"),
                        ));
                    });
                }
            }
            _ => {}
        }
    }
    results
}

pub(super) fn unique_wait(
    concealed: &[MahjongTileKind],
    melds: &[Meld],
    winning: MahjongTileKind,
) -> bool {
    let mut before = concealed.to_vec();
    let Some(position) = before.iter().position(|tile| *tile == winning) else {
        return false;
    };
    before.remove(position);
    let mut wins = 0;
    for index in 0..34 {
        let tile = MahjongTileKind::from_index34(index).expect("valid index");
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
