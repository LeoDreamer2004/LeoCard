use super::*;
use crate::{
    MahjongPlayerId, MahjongRuleSet, MahjongTile, MahjongTileKind, MahjongWind, Meld, build_deck,
};
use std::collections::HashSet;

fn finish_dealing(game: &mut GameState) {
    while matches!(game.phase(), Phase::Dealing { .. }) {
        game.advance_deal().unwrap();
    }
}

#[test]
fn developer_hand_replacement_swaps_only_with_wall_tiles() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    finish_dealing(&mut game);
    let player = MahjongPlayerId(1);
    let old_hand = game.players[player.0].hand.clone();
    let target = game
        .wall
        .iter()
        .copied()
        .find(|tile| {
            !tile.kind().is_flower() && old_hand.iter().all(|held| held.kind() != tile.kind())
        })
        .unwrap();
    let before_pool = old_hand
        .iter()
        .chain(game.wall.iter())
        .copied()
        .collect::<HashSet<_>>();
    let wall_len = game.wall.len();
    let mut kinds = old_hand.iter().map(|tile| tile.kind()).collect::<Vec<_>>();
    kinds[0] = target.kind();

    game.replace_player_hand_from_wall(player, &kinds).unwrap();

    assert_eq!(game.wall.len(), wall_len);
    assert_eq!(
        game.players[player.0]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect::<Vec<_>>(),
        kinds
    );
    assert!(game.players[player.0].hand.contains(&target));
    assert!(!game.wall.contains(&target));
    assert_eq!(
        game.players[player.0]
            .hand
            .iter()
            .chain(game.wall.iter())
            .copied()
            .collect::<HashSet<_>>(),
        before_pool
    );
}

#[test]
fn invalid_developer_hand_replacement_is_atomic() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    finish_dealing(&mut game);
    let player = MahjongPlayerId(1);

    let before = game.clone();
    let too_short = game.players[player.0].hand[1..]
        .iter()
        .map(|tile| tile.kind())
        .collect::<Vec<_>>();
    assert_eq!(
        game.replace_player_hand_from_wall(player, &too_short),
        Err(GameError::InvalidHandReplacement(
            MahjongHandReplacementError::WrongTileCount {
                expected: 13,
                actual: 12,
            }
        ))
    );
    assert_eq!(game, before);

    let unavailable =
        vec![MahjongTileKind::Wind(MahjongWind::East); game.players[player.0].hand.len()];
    assert_eq!(
        game.replace_player_hand_from_wall(player, &unavailable),
        Err(GameError::InvalidHandReplacement(
            MahjongHandReplacementError::TileUnavailable {
                tile: MahjongTileKind::Wind(MahjongWind::East),
                requested: 13,
                available: 4,
            }
        ))
    );
    assert_eq!(game, before);
}

#[test]
fn single_hand_ready_continues_round_rotation_but_resets_match_score() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(2))
            .unwrap();
    finish_dealing(&mut game);
    game.finish_exhaustive_draw().unwrap();
    game.start_next_hand(build_deck()).unwrap();
    assert!(matches!(game.phase(), Phase::Dealing { batch: 0 }));
    assert_eq!(game.sequence_index(), 1);
    assert_eq!(game.dealer(), MahjongPlayerId(3));
    assert_eq!(game.prevalent_wind(), MahjongWind::East);
    assert_eq!(game.match_scores(), &[0; 4]);
}

#[test]
fn false_win_penalty_reveals_and_kills_the_hand() {
    let mut game = GameState::new_with_deck(
        MahjongRuleSet {
            false_win: true,
            ..MahjongRuleSet::default()
        },
        build_deck(),
        MahjongPlayerId(0),
    )
    .unwrap();
    finish_dealing(&mut game);
    let delta = game.apply_false_win(MahjongPlayerId(1));
    assert_eq!(delta, [10, -30, 10, 10]);
    assert!(game.players[1].dead_hand);
    assert!(
        game.public_player(MahjongPlayerId(0), MahjongPlayerId(1))
            .unwrap()
            .revealed_hand
            .is_some()
    );
}

#[test]
fn concealed_kong_kind_is_hidden_from_other_players_until_settlement() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    finish_dealing(&mut game);
    game.players[0]
        .melds
        .push(Meld::concealed_kong(MahjongTileKind::Dragon(
            crate::MahjongDragon::White,
        )));
    let owner_view = game
        .public_player(MahjongPlayerId(0), MahjongPlayerId(0))
        .unwrap();
    let other_view = game
        .public_player(MahjongPlayerId(1), MahjongPlayerId(0))
        .unwrap();
    assert_eq!(
        owner_view.melds.last().unwrap().tile,
        Some(MahjongTileKind::Dragon(crate::MahjongDragon::White))
    );
    assert_eq!(other_view.melds.last().unwrap().tile, None);
    game.finish_exhaustive_draw().unwrap();
    assert_eq!(
        game.public_player(MahjongPlayerId(1), MahjongPlayerId(0))
            .unwrap()
            .melds
            .last()
            .unwrap()
            .tile,
        Some(MahjongTileKind::Dragon(crate::MahjongDragon::White))
    );
}

#[test]
fn a_chow_waits_for_a_possible_higher_priority_pung() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    finish_dealing(&mut game);
    for player in &mut game.players {
        player.hand.clear();
    }
    game.players[1].hand.extend([
        MahjongTile::new(
            MahjongTileKind::suited(crate::MahjongSuit::Characters, 2),
            0,
        ),
        MahjongTile::new(
            MahjongTileKind::suited(crate::MahjongSuit::Characters, 3),
            0,
        ),
    ]);
    game.players[2].hand.extend([
        MahjongTile::new(
            MahjongTileKind::suited(crate::MahjongSuit::Characters, 1),
            1,
        ),
        MahjongTile::new(
            MahjongTileKind::suited(crate::MahjongSuit::Characters, 1),
            2,
        ),
    ]);
    let discarded = MahjongTile::new(
        MahjongTileKind::suited(crate::MahjongSuit::Characters, 1),
        0,
    );
    let pending = game
        .pending_for_discard(0, MahjongPlayerId(0), discarded)
        .unwrap();
    assert!(
        pending
            .options_for(MahjongPlayerId(1))
            .unwrap()
            .contains(&MahjongClaimOption::Chow { start: 1 })
    );
    assert!(
        pending
            .options_for(MahjongPlayerId(2))
            .unwrap()
            .contains(&MahjongClaimOption::Pung)
    );
    assert_eq!(
        pending.waiting_for(),
        vec![MahjongPlayerId(1), MahjongPlayerId(2)]
    );
}

fn game_waiting_for_win_pung_and_chow() -> GameState {
    let mut game = GameState::new_with_deck(
        MahjongRuleSet {
            minimum_eight_points: false,
            ..MahjongRuleSet::default()
        },
        build_deck(),
        MahjongPlayerId(0),
    )
    .unwrap();
    finish_dealing(&mut game);
    for player in &mut game.players {
        player.hand.clear();
    }
    let character = |rank| MahjongTileKind::suited(crate::MahjongSuit::Characters, rank);
    game.players[1].hand.extend([
        MahjongTile::new(character(2), 0),
        MahjongTile::new(character(3), 0),
    ]);
    game.players[2].hand.extend([
        MahjongTile::new(character(1), 1),
        MahjongTile::new(character(1), 2),
    ]);
    game.players[3].hand.extend([
        MahjongTile::new(character(2), 1),
        MahjongTile::new(character(3), 1),
        MahjongTile::new(character(4), 0),
        MahjongTile::new(character(5), 0),
        MahjongTile::new(character(6), 0),
        MahjongTile::new(character(7), 0),
        MahjongTile::new(character(8), 0),
        MahjongTile::new(character(9), 0),
        MahjongTile::new(MahjongTileKind::Wind(MahjongWind::East), 0),
        MahjongTile::new(MahjongTileKind::Wind(MahjongWind::East), 1),
        MahjongTile::new(MahjongTileKind::Wind(MahjongWind::East), 2),
        MahjongTile::new(MahjongTileKind::Dragon(crate::MahjongDragon::Red), 0),
        MahjongTile::new(MahjongTileKind::Dragon(crate::MahjongDragon::Red), 1),
    ]);
    let tile = MahjongTile::new(character(1), 0);
    game.discards.push(Discard {
        player: MahjongPlayerId(0),
        tile,
        claimed_by: None,
    });
    game.phase = Phase::WaitingForClaims(
        game.pending_for_discard(0, MahjongPlayerId(0), tile)
            .unwrap(),
    );
    game
}

#[test]
fn highest_priority_claim_skips_all_lower_players() {
    let mut game = game_waiting_for_win_pung_and_chow();
    let outcome = game
        .respond_to_claim(MahjongPlayerId(3), MahjongClaim::Win)
        .unwrap();
    let ActionOutcome::HandFinished(result) = outcome else {
        panic!("the winning claim should resolve immediately");
    };
    assert_eq!(result.winners.len(), 1);
    assert_eq!(result.winners[0].player, MahjongPlayerId(3));
}

#[test]
fn next_claim_resolves_as_soon_as_higher_player_passes() {
    let mut game = game_waiting_for_win_pung_and_chow();
    assert_eq!(
        game.respond_to_claim(MahjongPlayerId(3), MahjongClaim::Pass)
            .unwrap(),
        ActionOutcome::ClaimRecorded {
            player: MahjongPlayerId(3)
        }
    );
    let outcome = game
        .respond_to_claim(MahjongPlayerId(2), MahjongClaim::Pung)
        .unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::Claimed {
            player: MahjongPlayerId(2),
            claim: MahjongClaim::Pung,
            ..
        }
    ));
}
