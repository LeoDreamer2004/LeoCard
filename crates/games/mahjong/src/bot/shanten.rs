use std::collections::HashMap;

pub(super) type Counts = [u8; 34];
type Shapes = Vec<[u8; 3]>;

#[derive(Default)]
pub(super) struct Shanten {
    suits: HashMap<u32, Shapes>,
    hands: HashMap<(Counts, u8), i8>,
}

impl Shanten {
    pub fn calculate(&mut self, counts: &Counts, fixed: u8) -> i8 {
        if let Some(value) = self.hands.get(&(*counts, fixed)) {
            return *value;
        }
        let mut best = self.regular(counts, fixed);
        if fixed == 0 {
            // MCR permits a four-of-a-kind to supply two pairs.
            let pairs = counts.iter().map(|count| count / 2).sum::<u8>();
            best = best.min(6 - pairs as i8);
            let orphans = [0, 8, 9, 17, 18, 26, 27, 28, 29, 30, 31, 32, 33];
            let unique = orphans.iter().filter(|index| counts[**index] > 0).count();
            let pair = orphans.iter().any(|index| counts[*index] >= 2);
            best = best.min(13 - unique as i8 - i8::from(pair));
        }
        if fixed <= 1 {
            for permutation in permutations() {
                let mut remaining = *counts;
                let mut missing = 0;
                let mut unique = counts[27..].iter().filter(|count| **count > 0).count();
                for (group, suit) in permutation.into_iter().enumerate() {
                    for rank in [group, group + 3, group + 6] {
                        let index = suit * 9 + rank;
                        if remaining[index] > 0 {
                            remaining[index] -= 1;
                            unique += 1;
                        } else {
                            missing += 1;
                        }
                    }
                }
                best = best.min(missing + self.regular(&remaining, fixed + 3));
                if fixed == 0 {
                    best = best.min(13 - unique as i8);
                }
            }
        }
        self.hands.insert((*counts, fixed), best);
        best
    }

    pub fn regular(&mut self, counts: &Counts, fixed: u8) -> i8 {
        let mut shapes = vec![[fixed, 0, 0]];
        for group in 0..4 {
            let mut tiles = [0; 9];
            if group < 3 {
                tiles.copy_from_slice(&counts[group * 9..group * 9 + 9]);
            } else {
                tiles[..7].copy_from_slice(&counts[27..]);
            }
            let key = tiles
                .iter()
                .rev()
                .fold(0, |key, count| key * 5 + u32::from(*count))
                + if group == 3 { 1 << 24 } else { 0 };
            let group_shapes = self.suits.entry(key).or_insert_with(|| {
                let mut table = [[None; 2]; 5];
                decompose(&mut tiles, group != 3, [0; 3], &mut table);
                table
                    .iter()
                    .enumerate()
                    .flat_map(|(melds, pairs)| {
                        pairs.iter().enumerate().filter_map(move |(pair, blocks)| {
                            blocks.map(|blocks| [melds as u8, blocks, pair as u8])
                        })
                    })
                    .collect()
            });
            let mut table = [[None; 2]; 5];
            for left in &shapes {
                for right in group_shapes.iter() {
                    let melds = left[0] + right[0];
                    let pair = left[2] + right[2];
                    if melds <= 4 && pair <= 1 {
                        let blocks = (left[1] + right[1]).min(4 - melds);
                        let value = &mut table[usize::from(melds)][usize::from(pair)];
                        *value = Some(value.unwrap_or(0).max(blocks));
                    }
                }
            }
            shapes = table
                .iter()
                .enumerate()
                .flat_map(|(melds, pairs)| {
                    pairs.iter().enumerate().filter_map(move |(pair, blocks)| {
                        blocks.map(|blocks| [melds as u8, blocks, pair as u8])
                    })
                })
                .collect();
        }
        shapes
            .iter()
            .map(|shape| 8 - 2 * shape[0] as i8 - shape[1] as i8 - shape[2] as i8)
            .min()
            .unwrap_or(8)
    }
}

fn decompose(tiles: &mut [u8; 9], suited: bool, shape: [u8; 3], table: &mut [[Option<u8>; 2]; 5]) {
    let Some(index) = tiles.iter().position(|count| *count > 0) else {
        let slot = &mut table[usize::from(shape[0])][usize::from(shape[2])];
        *slot = Some(slot.unwrap_or(0).max(shape[1]));
        return;
    };
    let unused = tiles[index];
    tiles[index] = 0;
    decompose(tiles, suited, shape, table);
    tiles[index] = unused;
    if shape[0] < 4 && tiles[index] >= 3 {
        tiles[index] -= 3;
        decompose(tiles, suited, [shape[0] + 1, shape[1], shape[2]], table);
        tiles[index] += 3;
    }
    if suited && index < 7 && shape[0] < 4 && tiles[index + 1] > 0 && tiles[index + 2] > 0 {
        for tile in &mut tiles[index..=index + 2] {
            *tile -= 1;
        }
        decompose(tiles, suited, [shape[0] + 1, shape[1], shape[2]], table);
        for tile in &mut tiles[index..=index + 2] {
            *tile += 1;
        }
    }
    if tiles[index] >= 2 {
        tiles[index] -= 2;
        if shape[2] == 0 {
            decompose(tiles, suited, [shape[0], shape[1], 1], table);
        }
        if shape[1] < 4 {
            decompose(tiles, suited, [shape[0], shape[1] + 1, shape[2]], table);
        }
        tiles[index] += 2;
    }
    if suited && shape[1] < 4 {
        for offset in 1..=2 {
            if index + offset < 9 && tiles[index + offset] > 0 {
                tiles[index] -= 1;
                tiles[index + offset] -= 1;
                decompose(tiles, suited, [shape[0], shape[1] + 1, shape[2]], table);
                tiles[index] += 1;
                tiles[index + offset] += 1;
            }
        }
    }
}

pub(super) fn permutations() -> [[usize; 3]; 6] {
    [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
}
