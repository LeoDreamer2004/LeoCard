use super::helpers::{game, suited};
use crate::{
    MahjongFan, MahjongGameError, MahjongMeld, MahjongSuit, MahjongTile, MahjongTileKind,
    MahjongWinSource, MahjongWind,
};
use leocard_game_common::PlayerId;

#[test]
fn declaration_counts_toward_eight_and_locks_the_hand() {
    let mut hand = suited(MahjongSuit::Characters, &[1, 2]);
    hand.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    hand.extend(suited(MahjongSuit::Bamboo, &[9]));
    let melds = vec![
        MahjongMeld::chow(MahjongSuit::Dots, 2, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Bamboo, 3, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Dots, 6, PlayerId(1)),
    ];
    let mut game = game(hand, melds);
    let winning = MahjongTileKind::suited(MahjongSuit::Characters, 3);
    let discarded = game.players[0].hand.pop().unwrap();
    let before = game
        .score_for(PlayerId(0), winning, MahjongWinSource::Discard(PlayerId(1)))
        .unwrap();
    assert_eq!(before.points_without_flowers, 7);
    game.players[0].hand.push(discarded);
    let discard = game.last_drawn().unwrap();
    assert!(game.declared_ready_discards(PlayerId(0)).contains(&discard));
    game.declare_ready(PlayerId(0), discard).unwrap();
    let score = game
        .score_for(PlayerId(0), winning, MahjongWinSource::Discard(PlayerId(1)))
        .unwrap();
    assert_eq!(score.points_without_flowers, 8);
    assert!(
        score
            .fans
            .iter()
            .any(|fan| fan.fan == MahjongFan::DeclaredReady)
    );
    game.current_player = PlayerId(0);
    let drawn = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Dots, 9), 0);
    game.players[0].hand.push(drawn);
    game.last_drawn = Some(drawn);
    let held = game.players[0].hand[0];
    let before = game.clone();
    assert_eq!(
        game.discard(PlayerId(0), held),
        Err(MahjongGameError::ReadyHandLocked)
    );
    assert_eq!(game, before);
    assert!(game.discard(PlayerId(0), drawn).is_ok());
    game.finish_exhaustive_draw().unwrap();
    game.start_next_hand(crate::build_deck()).unwrap();
    assert!(!game.players[0].has_declared_ready());
}

#[test]
fn disabled_rule_cannot_declare_and_does_not_change_the_hand() {
    let hand = suited(
        MahjongSuit::Characters,
        &[1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 9],
    );
    let mut game = game(hand, Vec::new());
    let tile = game.last_drawn().unwrap();
    assert!(game.declared_ready_discards(PlayerId(0)).contains(&tile));
    game.rules.declared_ready = false;
    let before = game.clone();
    assert!(game.declared_ready_discards(PlayerId(0)).is_empty());
    assert_eq!(
        game.declare_ready(PlayerId(0), tile),
        Err(MahjongGameError::CannotDeclareReady)
    );
    assert_eq!(game, before);
}

#[test]
fn chicken_hand_is_replaced_by_one_ready_fan_and_no_longer_meets_the_minimum() {
    let mut hand = suited(MahjongSuit::Characters, &[5, 6]);
    hand.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    hand.extend(suited(MahjongSuit::Characters, &[9]));
    let melds = vec![
        MahjongMeld::chow(MahjongSuit::Characters, 2, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Dots, 5, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Bamboo, 7, PlayerId(1)),
    ];
    let mut game = game(hand, melds);
    let discard = game.last_drawn().unwrap();
    game.players[0].hand.pop();
    let winning = MahjongTileKind::suited(MahjongSuit::Characters, 4);
    let score = game
        .score_for(PlayerId(0), winning, MahjongWinSource::Discard(PlayerId(1)))
        .unwrap();
    assert_eq!(score.points_without_flowers, 8);
    assert!(
        score
            .fans
            .iter()
            .any(|fan| fan.fan == MahjongFan::ChickenHand)
    );
    game.players[0].hand.push(discard);
    assert!(game.declared_ready_discards(PlayerId(0)).contains(&discard));
    game.declare_ready(PlayerId(0), discard).unwrap();
    let score = game
        .score_for(PlayerId(0), winning, MahjongWinSource::Discard(PlayerId(1)))
        .unwrap();
    assert_eq!(score.points_without_flowers, 1);
    assert!(!game.is_legal_score(&score));
    assert!(
        !score
            .fans
            .iter()
            .any(|fan| fan.fan == MahjongFan::ChickenHand)
    );
}
