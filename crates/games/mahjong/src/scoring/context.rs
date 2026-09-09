use super::patterns::{
    all_tile_kinds, concealed_pung_count, has_chow_starts_same_suit, has_mixed_shifted_pungs,
    has_mixed_straight, is_pure_terminal_chows, is_three_suited_terminal_chows, kind_counts,
    max_matching, pure_shifted_chow_count, pure_shifted_pung_count, same_chow_count,
    set_has_terminal_or_honor, suit_summary,
};
use super::{Form, ScoreInput, Set, SetKind, WinSource};
use crate::{MahjongKongKind, MahjongMeldKind, MahjongSuit, MahjongTileKind};

pub(super) struct ScoreContext<'a> {
    pub(super) input: &'a ScoreInput,
    pub(super) form: &'a Form,
    pub(super) all_tiles: Vec<MahjongTileKind>,
    pub(super) sets_pair: Option<(&'a [Set], MahjongTileKind)>,
    pub(super) is_standard: bool,
    pub(super) pungs: Vec<MahjongTileKind>,
    pub(super) chows: Vec<(MahjongSuit, u8)>,
    pub(super) kongs: Vec<MahjongKongKind>,
    pub(super) wind_pungs: usize,
    pub(super) dragon_pungs: usize,
    pub(super) big_four: bool,
    pub(super) big_three_dragons: bool,
    pub(super) little_four: bool,
    pub(super) little_three: bool,
    pub(super) all_terminals: bool,
    pub(super) all_honors: bool,
    pub(super) concealed_pungs: usize,
    pub(super) pure_terminal_chows: bool,
    pub(super) quadruple_chow: bool,
    pub(super) four_shifted_pungs: bool,
    pub(super) four_shifted_chows: bool,
    pub(super) terminals_honors: bool,
    pub(super) all_even_pungs: bool,
    pub(super) suit_count: usize,
    pub(super) has_honor: bool,
    pub(super) full_flush: bool,
    pub(super) pure_triple: bool,
    pub(super) pure_shifted_pungs: bool,
    pub(super) pure_straight: bool,
    pub(super) three_suited_terminal: bool,
    pub(super) pure_shifted_chows: bool,
    pub(super) triple_pung: bool,
    pub(super) mixed_straight: bool,
    pub(super) mixed_triple: bool,
    pub(super) mixed_shifted_pungs: bool,
    pub(super) all_pungs: bool,
    pub(super) all_types: bool,
    pub(super) melded_hand: bool,
    pub(super) outside: bool,
    pub(super) fully_concealed: bool,
    pub(super) concealed_hand: bool,
    pub(super) all_chows: bool,
    pub(super) tile_hogs: u8,
    pub(super) double_pungs: u8,
    pub(super) all_simples: bool,
}

impl<'a> ScoreContext<'a> {
    pub(super) fn new(input: &'a ScoreInput, form: &'a Form) -> Self {
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
        let kongs = input
            .melds
            .iter()
            .filter_map(|meld| match meld.kind() {
                MahjongMeldKind::Kong(kind) => Some(kind),
                _ => None,
            })
            .collect::<Vec<_>>();
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
        let little_three = sets_pair.is_some_and(|(_, pair)| {
            dragon_pungs == 2 && matches!(pair, MahjongTileKind::Dragon(_))
        });
        let all_terminals = all_tiles.iter().all(|tile| tile.is_terminal());
        let all_honors = all_tiles.iter().all(|tile| tile.is_honor());
        let concealed_pungs = concealed_pung_count(input, sets_pair.map_or(&[], |value| value.0));
        let pure_terminal_chows =
            sets_pair.is_some_and(|(sets, pair)| is_pure_terminal_chows(sets, pair));
        let quadruple_chow = same_chow_count(&chows, 4);
        let four_shifted_pungs = pure_shifted_pung_count(&pungs, 4);
        let four_shifted_chows = pure_shifted_chow_count(&chows, 4);
        let terminals_honors = all_tiles.iter().all(|tile| tile.is_terminal_or_honor())
            && all_tiles.iter().any(|tile| tile.is_terminal())
            && all_tiles.iter().any(|tile| tile.is_honor());
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
        let (suit_count, has_honor) = suit_summary(&all_tiles);
        let full_flush = suit_count == 1 && !has_honor;
        let pure_triple = same_chow_count(&chows, 3);
        let pure_shifted_pungs = pure_shifted_pung_count(&pungs, 3);
        let pure_straight = has_chow_starts_same_suit(&chows, &[1, 4, 7]);
        let three_suited_terminal =
            sets_pair.is_some_and(|(sets, pair)| is_three_suited_terminal_chows(sets, pair));
        let pure_shifted_chows = pure_shifted_chow_count(&chows, 3);
        let triple_pung = (1..=9).any(|rank| {
            MahjongSuit::ALL
                .iter()
                .all(|suit| pungs.contains(&MahjongTileKind::suited(*suit, rank)))
        });
        let mixed_straight = has_mixed_straight(&chows);
        let mixed_triple = (1..=7).any(|start| {
            MahjongSuit::ALL
                .iter()
                .all(|suit| chows.contains(&(*suit, start)))
        });
        let mixed_shifted_pungs = has_mixed_shifted_pungs(&pungs);
        let all_pungs = sets_pair.is_some_and(|(sets, _)| {
            is_standard && sets.iter().all(|set| matches!(set.kind, SetKind::Pung(_)))
        });
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
        let melded_hand = sets_pair.is_some_and(|(sets, _)| {
            sets.len() == 4
                && sets.iter().all(|set| set.open)
                && matches!(input.context.source, WinSource::Discard(_))
        });
        let outside = sets_pair.is_some_and(|(sets, pair)| {
            is_standard && pair.is_terminal_or_honor() && sets.iter().all(set_has_terminal_or_honor)
        });
        let fully_concealed =
            input.melds.iter().all(|meld| !meld.is_open()) && input.context.source.is_self_draw();
        let concealed_hand = input.melds.iter().all(|meld| !meld.is_open())
            && matches!(
                input.context.source,
                WinSource::Discard(_) | WinSource::RobbingKong(_)
            );
        let all_chows = sets_pair.is_some_and(|(sets, pair)| {
            !pair.is_honor()
                && sets
                    .iter()
                    .all(|set| matches!(set.kind, SetKind::Chow { .. }))
                && (is_standard || matches!(form, Form::Knitted { straight: true, .. }))
        });
        let kong_tiles = input
            .melds
            .iter()
            .filter(|meld| matches!(meld.kind(), MahjongMeldKind::Kong(_)))
            .map(|meld| meld.tile())
            .collect::<Vec<_>>();
        let tile_hogs = counts
            .iter()
            .enumerate()
            .filter(|(index, count)| {
                **count == 4
                    && !kong_tiles
                        .contains(&MahjongTileKind::from_index34(*index).expect("valid index"))
            })
            .count() as u8;
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
        let all_simples = all_tiles.iter().all(|tile| !tile.is_terminal_or_honor());
        Self {
            input,
            form,
            all_tiles,
            sets_pair,
            is_standard,
            pungs,
            chows,
            kongs,
            wind_pungs,
            dragon_pungs,
            big_four,
            big_three_dragons,
            little_four,
            little_three,
            all_terminals,
            all_honors,
            concealed_pungs,
            pure_terminal_chows,
            quadruple_chow,
            four_shifted_pungs,
            four_shifted_chows,
            terminals_honors,
            all_even_pungs,
            suit_count,
            has_honor,
            full_flush,
            pure_triple,
            pure_shifted_pungs,
            pure_straight,
            three_suited_terminal,
            pure_shifted_chows,
            triple_pung,
            mixed_straight,
            mixed_triple,
            mixed_shifted_pungs,
            all_pungs,
            all_types,
            melded_hand,
            outside,
            fully_concealed,
            concealed_hand,
            all_chows,
            tile_hogs,
            double_pungs,
            all_simples,
        }
    }
}
