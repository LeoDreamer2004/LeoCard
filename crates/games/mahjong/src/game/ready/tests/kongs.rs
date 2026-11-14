use super::helpers::{game, suited};
use crate::{
    MahjongClaim, MahjongClaimOption, MahjongGameError, MahjongMeld, MahjongPhase, MahjongSuit,
    MahjongTile, MahjongTileKind, MahjongWind, waiting_tile_kinds,
};
use leocard_game_common::PlayerId;

#[test]
fn concealed_and_direct_kongs_cannot_shrink_the_declared_waits() {
    let hand = suited(MahjongSuit::Characters, &[3, 4, 5, 6, 7, 7, 7]);
    let melds = vec![
        MahjongMeld::chow(MahjongSuit::Dots, 1, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Bamboo, 1, PlayerId(1)),
    ];
    let waits = waiting_tile_kinds(&hand, &melds);
    assert_eq!(waits, suited(MahjongSuit::Characters, &[2, 3, 5, 6, 8]));
    let mut game = game(hand, melds);
    game.players[0].ready_waits = Some(waits);
    let tile = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, 7), 3);
    let pending = game.pending_for_discard(0, PlayerId(1), tile).unwrap();
    assert!(
        !pending
            .options_for(PlayerId(0))
            .unwrap()
            .contains(&MahjongClaimOption::Kong)
    );
    game.players[0].hand.push(tile);
    game.last_drawn = Some(tile);
    assert!(!game.concealed_kong_available(PlayerId(0), tile.kind()));
    let before = game.clone();
    assert_eq!(
        game.declare_concealed_kong(PlayerId(0), tile.kind()),
        Err(MahjongGameError::CannotKong)
    );
    assert_eq!(game, before);
}

#[test]
fn all_three_kong_types_keep_an_identical_wait_and_replace_from_the_tail() {
    for mode in 0..3 {
        let kind = MahjongTileKind::suited(MahjongSuit::Characters, 1);
        let mut hand = suited(MahjongSuit::Dots, &[2, 3, 4, 4, 5, 6]);
        hand.extend(suited(MahjongSuit::Bamboo, &[7, 8, 9]));
        hand.push(MahjongTileKind::Wind(MahjongWind::East));
        let mut melds = Vec::new();
        if mode == 1 {
            melds.push(MahjongMeld::pung(kind, PlayerId(1)));
        } else {
            hand.extend([kind; 3]);
        }
        let waits = waiting_tile_kinds(&hand, &melds);
        assert_eq!(waits, vec![MahjongTileKind::Wind(MahjongWind::East)]);
        let mut game = game(hand, melds);
        game.players[0].ready_waits = Some(waits.clone());
        let fourth = MahjongTile::new(kind, 3);
        let tail = game.wall_tail_drawn();
        if mode == 2 {
            let pending = game.pending_for_discard(0, PlayerId(1), fourth).unwrap();
            let options = pending.options_for(PlayerId(0)).unwrap();
            assert!(options.contains(&MahjongClaimOption::Kong));
            assert!(!options.contains(&MahjongClaimOption::Pung));
            game.discards.push(crate::MahjongDiscard {
                player: PlayerId(1),
                tile: fourth,
                claimed_by: None,
                from_drawn: true,
            });
            game.phase = MahjongPhase::WaitingForClaims(pending);
            game.respond_to_claim(PlayerId(0), MahjongClaim::Kong)
                .unwrap();
        } else {
            game.players[0].hand.push(fourth);
            game.last_drawn = Some(fourth);
            if mode == 0 {
                game.declare_concealed_kong(PlayerId(0), kind).unwrap();
            } else {
                game.declare_added_kong(PlayerId(0), fourth).unwrap();
            }
        }
        assert_eq!(game.wall_tail_drawn(), tail + 1);
        assert!(game.players[0].has_declared_ready());
        let drawn = game.last_drawn().unwrap();
        let mut concealed = game.players[0].hand.clone();
        concealed.remove(concealed.iter().position(|tile| *tile == drawn).unwrap());
        assert_eq!(
            waiting_tile_kinds(
                &concealed.iter().map(|tile| tile.kind()).collect::<Vec<_>>(),
                &game.players[0].melds
            ),
            waits
        );
    }
}

#[test]
fn added_kong_cannot_replace_the_declared_waits_with_new_ones() {
    let kind = MahjongTileKind::suited(MahjongSuit::Characters, 7);
    let mut hand = suited(MahjongSuit::Characters, &[3, 4, 5, 6, 7]);
    hand.extend(suited(MahjongSuit::Dots, &[2, 3, 4]));
    hand.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    let melds = vec![MahjongMeld::pung(kind, PlayerId(1))];
    let waits = waiting_tile_kinds(&hand, &melds);
    assert_eq!(waits, suited(MahjongSuit::Characters, &[2, 5, 8]));
    let mut game = game(hand, melds);
    game.players[0].ready_waits = Some(waits);
    let drawn = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, 6), 1);
    game.players[0].hand.push(drawn);
    game.last_drawn = Some(drawn);
    let seventh = *game.players[0]
        .hand
        .iter()
        .find(|tile| tile.kind() == kind)
        .unwrap();
    assert!(!game.added_kong_available(PlayerId(0), seventh));
    let before = game.clone();
    assert_eq!(
        game.declare_added_kong(PlayerId(0), seventh),
        Err(MahjongGameError::CannotKong)
    );
    assert_eq!(game, before);
}
