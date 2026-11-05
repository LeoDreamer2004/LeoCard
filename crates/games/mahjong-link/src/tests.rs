use super::*;
use leocard_mahjong_common::{MahjongSuit, MahjongTileKind, MahjongWind};

mod exposed;
mod turns;

fn tile(rank: u8) -> MahjongTileKind {
    MahjongTileKind::suited(MahjongSuit::Characters, rank)
}

#[test]
fn deck_and_rectangle_contain_all_136_tiles() {
    let board = MahjongLinkBoard::new(build_deck()).unwrap();
    assert_eq!(board.cells().len(), 136);
    assert_eq!(board.remaining(), 136);
    assert!(board.cells().iter().all(Option::is_some));
    assert_eq!(board.maximum_push(0, PushSide::Left), 0);
    assert_eq!(board.maximum_push(0, PushSide::Right), 0);
    let mut invalid = build_deck();
    invalid[0] = tile(9);
    assert_eq!(
        MahjongLinkBoard::new(invalid),
        Err(MahjongLinkError::InvalidDeck)
    );
}

#[test]
fn links_accept_zero_one_and_two_turns_including_outer_border() {
    let board = MahjongLinkBoard::fixture(&[(0, 0, tile(1)), (0, 3, tile(1)), (0, 1, tile(2))]);
    let path = board
        .link_path(
            LinkCell { row: 0, column: 0 },
            LinkCell { row: 0, column: 3 },
        )
        .unwrap();
    assert!(path.iter().any(|point| point.row == -1));
    let board = MahjongLinkBoard::fixture(&[(2, 2, tile(1)), (4, 4, tile(1))]);
    assert!(
        board
            .link_path(
                LinkCell { row: 2, column: 2 },
                LinkCell { row: 4, column: 4 }
            )
            .is_some()
    );
    let board = MahjongLinkBoard::fixture(&[(2, 2, tile(1)), (2, 3, tile(1))]);
    assert!(
        board
            .link_path(
                LinkCell { row: 2, column: 2 },
                LinkCell { row: 2, column: 3 }
            )
            .is_some()
    );
}

#[test]
fn blocked_and_different_tiles_cannot_be_removed() {
    let mut board = MahjongLinkBoard::fixture(&[
        (3, 3, tile(1)),
        (6, 6, tile(1)),
        (2, 3, tile(2)),
        (4, 3, tile(2)),
        (3, 2, tile(2)),
        (3, 4, tile(2)),
    ]);
    let original = board.clone();
    assert_eq!(
        board.remove_pair(
            LinkCell { row: 3, column: 3 },
            LinkCell { row: 6, column: 6 }
        ),
        Err(MahjongLinkError::PathBlocked)
    );
    assert_eq!(
        board.remove_pair(
            LinkCell { row: 3, column: 3 },
            LinkCell { row: 3, column: 2 }
        ),
        Err(MahjongLinkError::TilesDoNotMatch)
    );
    assert_eq!(board, original);
}

#[test]
fn pushing_from_row_end_collides_and_absorbs_gaps_at_any_distance() {
    let mut board = MahjongLinkBoard::fixture(&[
        (2, 1, tile(1)),
        (2, 4, tile(2)),
        (2, 5, tile(3)),
        (3, 4, tile(9)),
    ]);
    assert_eq!(board.maximum_push(2, PushSide::Left), 13);
    board.push(2, PushSide::Left, 5).unwrap();
    assert_eq!(board.tile(LinkCell { row: 2, column: 6 }), Some(tile(1)));
    assert_eq!(board.tile(LinkCell { row: 2, column: 7 }), Some(tile(2)));
    assert_eq!(board.tile(LinkCell { row: 2, column: 8 }), Some(tile(3)));
    assert_eq!(board.tile(LinkCell { row: 3, column: 4 }), Some(tile(9)));
    let original = board.clone();
    assert_eq!(
        board.push(2, PushSide::Left, 9),
        Err(MahjongLinkError::PushOutOfBounds)
    );
    assert_eq!(board, original);
    board.push(2, PushSide::Right, 6).unwrap();
    assert_eq!(board.tile(LinkCell { row: 2, column: 0 }), Some(tile(1)));
    assert_eq!(board.tile(LinkCell { row: 2, column: 1 }), Some(tile(2)));
    assert_eq!(board.tile(LinkCell { row: 2, column: 2 }), Some(tile(3)));
    assert_eq!(board.remaining(), 4);
}

#[test]
fn scores_are_awarded_once_per_pair() {
    for (kind, expected) in [(tile(9), 9), (MahjongTileKind::Wind(MahjongWind::East), 10)] {
        let mut board = MahjongLinkBoard::fixture(&[(0, 0, kind), (0, 1, kind)]);
        let (points, _) = board
            .remove_pair(
                LinkCell { row: 0, column: 0 },
                LinkCell { row: 0, column: 1 },
            )
            .unwrap();
        assert_eq!(points, expected);
        assert_eq!(board.remaining(), 0);
    }
}

#[test]
fn turns_reject_other_players_without_advancing_and_all_passes_finish() {
    for count in MahjongLinkRuleSet::MIN_PLAYERS..=MahjongLinkRuleSet::MAX_PLAYERS {
        let mut game = MahjongLinkGameState::new(count, build_deck()).unwrap();
        assert_eq!(
            game.act(1, MahjongLinkAction::Pass),
            Err(if count == 1 {
                MahjongLinkError::InvalidPlayer
            } else {
                MahjongLinkError::NotPlayersTurn
            })
        );
        assert_eq!(game.turn(), 0);
        let pair = MahjongLinkAction::Match {
            first: LinkCell { row: 0, column: 1 },
            second: LinkCell { row: 0, column: 2 },
        };
        assert_eq!(game.act(0, pair).unwrap().points, 1);
        assert_eq!(game.scores()[0], 1);
        assert_eq!(game.current_player(), Some(1 % count));
        for index in 0..count {
            let player = (index + 1) % count;
            game.act(player, MahjongLinkAction::Pass).unwrap();
        }
        assert!(game.complete());
        assert_eq!(game.current_player(), None);
        assert_eq!(
            game.act(0, MahjongLinkAction::Pass),
            Err(MahjongLinkError::MatchComplete)
        );
    }
}

#[test]
fn clearing_a_full_board_finishes_and_preserves_the_total_pair_score() {
    let mut game = MahjongLinkGameState::new(2, build_deck()).unwrap();
    for _ in 0..68 {
        let (first, second) = game
            .board()
            .best_pair()
            .expect("ordered deck has an available pair");
        let player = game.current_player().unwrap();
        game.act(player, MahjongLinkAction::Match { first, second })
            .unwrap();
    }
    assert!(game.complete());
    assert_eq!(game.board().remaining(), 0);
    assert_eq!(game.scores().iter().sum::<u16>(), 410);
    assert_eq!(game.turn(), 68);
}

#[test]
fn player_count_must_fit_the_table() {
    for count in [0, 9, u8::MAX] {
        assert!(matches!(
            MahjongLinkGameState::new(count, build_deck()),
            Err(MahjongLinkError::InvalidPlayerCount)
        ));
    }
}
