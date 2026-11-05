use crate::{LinkCell, LinkMovement, LinkPoint, MahjongLinkError, PushSide};
use leocard_game_common::mahjong::{MahjongTileKind, build_deck as build_mahjong_deck};
use std::collections::VecDeque;
use std::ops::Range;

pub const BOARD_ROWS: usize = 8;
pub const BOARD_COLUMNS: usize = 17;
const PATH_COLUMNS: usize = BOARD_COLUMNS + 2;
const PATH_ROWS: usize = BOARD_ROWS + 2;
const DIRECTIONS: [(i8, i8); 4] = [(-1, 0), (0, 1), (1, 0), (0, -1)];

pub fn build_deck() -> Vec<MahjongTileKind> {
    build_mahjong_deck()
        .into_iter()
        .filter_map(|tile| (!tile.kind().is_flower()).then_some(tile.kind()))
        .collect()
}

pub const fn tile_points(tile: MahjongTileKind) -> u16 {
    match tile {
        MahjongTileKind::Suited { rank, .. } => rank as u16,
        MahjongTileKind::Wind(_) | MahjongTileKind::Dragon(_) => 10,
        MahjongTileKind::Flower(_) => 0,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongLinkBoard {
    cells: Vec<Option<MahjongTileKind>>,
}

impl MahjongLinkBoard {
    pub fn new(deck: Vec<MahjongTileKind>) -> Result<Self, MahjongLinkError> {
        let mut expected = build_deck();
        let mut actual = deck.clone();
        expected.sort_unstable();
        actual.sort_unstable();
        if actual != expected {
            return Err(MahjongLinkError::InvalidDeck);
        }
        let cells = deck.into_iter().map(Some).collect();
        Ok(Self { cells })
    }

    pub fn cells(&self) -> &[Option<MahjongTileKind>] {
        &self.cells
    }

    pub fn tile(&self, cell: LinkCell) -> Option<MahjongTileKind> {
        Self::index(cell).and_then(|index| self.cells[index])
    }

    pub fn remaining(&self) -> usize {
        self.cells.iter().flatten().count()
    }

    fn index(cell: LinkCell) -> Option<usize> {
        (usize::from(cell.row) < BOARD_ROWS && usize::from(cell.column) < BOARD_COLUMNS)
            .then_some(usize::from(cell.row) * BOARD_COLUMNS + usize::from(cell.column))
    }

    pub fn link_path(&self, first: LinkCell, second: LinkCell) -> Option<Vec<LinkPoint>> {
        if first == second || self.tile(first).is_none() || self.tile(first) != self.tile(second) {
            return None;
        }
        let start = (first.row as i8 + 1, first.column as i8 + 1);
        let end = (second.row as i8 + 1, second.column as i8 + 1);
        let state_index = |row: i8, column: i8, direction: usize| {
            (row as usize * PATH_COLUMNS + column as usize) * 4 + direction
        };
        let mut turns = vec![u8::MAX; PATH_ROWS * PATH_COLUMNS * 4];
        let mut previous = vec![None; turns.len()];
        let mut queue = VecDeque::new();
        for direction in 0..4 {
            let index = state_index(start.0, start.1, direction);
            turns[index] = 0;
            queue.push_back((start.0, start.1, direction));
        }
        while let Some((row, column, direction)) = queue.pop_front() {
            let current = state_index(row, column, direction);
            if (row, column) == end {
                let mut path = vec![LinkPoint {
                    row: row - 1,
                    column: column - 1,
                }];
                let mut cursor = current;
                while let Some(parent) = previous[cursor] {
                    let point = parent / 4;
                    path.push(LinkPoint {
                        row: (point / PATH_COLUMNS) as i8 - 1,
                        column: (point % PATH_COLUMNS) as i8 - 1,
                    });
                    cursor = parent;
                }
                path.reverse();
                return Some(path);
            }
            for (next_direction, (dr, dc)) in DIRECTIONS.iter().copied().enumerate() {
                let next_turns = turns[current] + u8::from(direction != next_direction);
                if next_turns > 2 {
                    continue;
                }
                let next = (row + dr, column + dc);
                if next.0 < 0
                    || next.1 < 0
                    || next.0 >= PATH_ROWS as i8
                    || next.1 >= PATH_COLUMNS as i8
                {
                    continue;
                }
                let on_board = next.0 > 0
                    && next.0 <= BOARD_ROWS as i8
                    && next.1 > 0
                    && next.1 <= BOARD_COLUMNS as i8;
                if next != end
                    && on_board
                    && self.cells[(next.0 as usize - 1) * BOARD_COLUMNS + next.1 as usize - 1]
                        .is_some()
                {
                    continue;
                }
                let index = state_index(next.0, next.1, next_direction);
                if next_turns < turns[index] {
                    turns[index] = next_turns;
                    previous[index] = Some(current);
                    queue.push_back((next.0, next.1, next_direction));
                }
            }
        }
        self.vertical_outer_path(first, second)
    }

    // Opposite exposed vertical ends are an explicit exception to the two-turn rule.
    fn vertical_outer_path(&self, first: LinkCell, second: LinkCell) -> Option<Vec<LinkPoint>> {
        let exposed = |cell: LinkCell, mut rows: Range<usize>| {
            rows.all(|row| {
                self.tile(LinkCell {
                    row: row as u8,
                    column: cell.column,
                })
                .is_none()
            })
        };
        let top = |cell: LinkCell| exposed(cell, 0..usize::from(cell.row));
        let bottom = |cell: LinkCell| exposed(cell, usize::from(cell.row) + 1..BOARD_ROWS);
        let (first_edge, second_edge) = if top(first) && bottom(second) {
            (-1, BOARD_ROWS as i8)
        } else if bottom(first) && top(second) {
            (BOARD_ROWS as i8, -1)
        } else {
            return None;
        };
        let side = if usize::from(first.column) + usize::from(second.column) < BOARD_COLUMNS - 1 {
            -1
        } else {
            BOARD_COLUMNS as i8
        };
        Some(vec![
            LinkPoint {
                row: first.row as i8,
                column: first.column as i8,
            },
            LinkPoint {
                row: first_edge,
                column: first.column as i8,
            },
            LinkPoint {
                row: first_edge,
                column: side,
            },
            LinkPoint {
                row: second_edge,
                column: side,
            },
            LinkPoint {
                row: second_edge,
                column: second.column as i8,
            },
            LinkPoint {
                row: second.row as i8,
                column: second.column as i8,
            },
        ])
    }

    pub fn remove_pair(
        &mut self,
        first: LinkCell,
        second: LinkCell,
    ) -> Result<(u16, Vec<LinkPoint>), MahjongLinkError> {
        let tile = self.tile(first).ok_or(MahjongLinkError::InvalidCell)?;
        if first == second || self.tile(second) != Some(tile) {
            return Err(MahjongLinkError::TilesDoNotMatch);
        }
        let path = self
            .link_path(first, second)
            .ok_or(MahjongLinkError::PathBlocked)?;
        self.cells[Self::index(first).expect("validated cell")] = None;
        self.cells[Self::index(second).expect("validated cell")] = None;
        Ok((tile_points(tile), path))
    }

    pub fn maximum_push(&self, row: u8, from: PushSide) -> u8 {
        let Some(line) = self
            .cells
            .as_chunks::<BOARD_COLUMNS>()
            .0
            .get(usize::from(row))
        else {
            return 0;
        };
        let count = line.iter().flatten().count();
        if count == 0 {
            return 0;
        }
        let endpoint = match from {
            PushSide::Left => line.iter().position(Option::is_some).expect("nonempty row"),
            PushSide::Right => {
                BOARD_COLUMNS
                    - 1
                    - line
                        .iter()
                        .rposition(Option::is_some)
                        .expect("nonempty row")
            }
        };
        (BOARD_COLUMNS - count - endpoint) as u8
    }

    pub fn push(
        &mut self,
        row: u8,
        from: PushSide,
        distance: u8,
    ) -> Result<Vec<LinkMovement>, MahjongLinkError> {
        if usize::from(row) >= BOARD_ROWS || distance == 0 {
            return Err(MahjongLinkError::InvalidPush);
        }
        let maximum = self.maximum_push(row, from);
        if distance > maximum {
            return Err(MahjongLinkError::PushOutOfBounds);
        }
        let offset = usize::from(row) * BOARD_COLUMNS;
        let line = &mut self.cells[offset..offset + BOARD_COLUMNS];
        // Work from the pushed end; mirrored indexing gives both sides identical collision rules.
        let mut ordered: Vec<_> = line
            .iter()
            .copied()
            .enumerate()
            .map(|(column, tile)| tile.map(|tile| (tile, column)))
            .collect();
        if from == PushSide::Right {
            ordered.reverse();
        }
        let endpoint = ordered
            .iter()
            .position(Option::is_some)
            .ok_or(MahjongLinkError::InvalidPush)?;
        for endpoint in endpoint..endpoint + usize::from(distance) {
            let gap = (endpoint + 1..BOARD_COLUMNS)
                .find(|index| ordered[*index].is_none())
                .ok_or(MahjongLinkError::PushOutOfBounds)?;
            for index in (endpoint..gap).rev() {
                ordered[index + 1] = ordered[index];
            }
            ordered[endpoint] = None;
        }
        if from == PushSide::Right {
            ordered.reverse();
        }
        let mut movements = Vec::new();
        for (column, tile) in ordered.into_iter().enumerate() {
            line[column] = tile.map(|(kind, _)| kind);
            if let Some((_, source)) = tile
                && source != column
            {
                movements.push(LinkMovement {
                    from: LinkCell {
                        row,
                        column: source as u8,
                    },
                    to: LinkCell {
                        row,
                        column: column as u8,
                    },
                });
            }
        }
        Ok(movements)
    }

    pub fn best_pair(&self) -> Option<(LinkCell, LinkCell)> {
        let mut best = None;
        let mut value = 0;
        for (index, tile) in self.cells.iter().copied().enumerate() {
            let Some(tile) = tile else { continue };
            if tile_points(tile) <= value {
                continue;
            }
            let first = LinkCell {
                row: (index / BOARD_COLUMNS) as u8,
                column: (index % BOARD_COLUMNS) as u8,
            };
            for other in index + 1..self.cells.len() {
                if self.cells[other] != Some(tile) {
                    continue;
                }
                let second = LinkCell {
                    row: (other / BOARD_COLUMNS) as u8,
                    column: (other % BOARD_COLUMNS) as u8,
                };
                if self.link_path(first, second).is_some() {
                    best = Some((first, second));
                    value = tile_points(tile);
                    break;
                }
            }
        }
        best
    }

    #[cfg(test)]
    pub(crate) fn fixture(tiles: &[(u8, u8, MahjongTileKind)]) -> Self {
        let mut cells = vec![None; BOARD_ROWS * BOARD_COLUMNS];
        for &(row, column, tile) in tiles {
            cells[usize::from(row) * BOARD_COLUMNS + usize::from(column)] = Some(tile);
        }
        Self { cells }
    }
}
