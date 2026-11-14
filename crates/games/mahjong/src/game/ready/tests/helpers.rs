use crate::{
    MahjongGameState, MahjongMeld, MahjongPhase, MahjongRuleSet, MahjongSuit, MahjongTile,
    MahjongTileKind, build_deck,
};
use leocard_game_common::PlayerId;
use std::collections::HashMap;

pub(super) fn suited(suit: MahjongSuit, ranks: &[u8]) -> Vec<MahjongTileKind> {
    ranks
        .iter()
        .map(|rank| MahjongTileKind::suited(suit, *rank))
        .collect()
}

pub(super) fn game(kinds: Vec<MahjongTileKind>, melds: Vec<MahjongMeld>) -> MahjongGameState {
    let mut game = MahjongGameState::new_with_deck(
        MahjongRuleSet {
            declared_ready: true,
            ..Default::default()
        },
        build_deck(),
        PlayerId(0),
    )
    .unwrap();
    let mut copies = HashMap::new();
    game.players[0].hand = kinds
        .into_iter()
        .map(|kind| {
            let copy = copies.entry(kind).or_insert(0);
            let tile = MahjongTile::new(kind, *copy);
            *copy += 1;
            tile
        })
        .collect();
    game.players[0].melds = melds;
    game.last_drawn = game.players[0].hand.last().copied();
    game.phase = MahjongPhase::Playing;
    game
}
