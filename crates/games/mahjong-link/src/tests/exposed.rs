use super::*;

#[test]
fn opposite_exposed_tiles_match_even_when_the_normal_route_needs_more_turns() {
    for (top_row, bottom_row) in [(0, 7), (1, 6)] {
        for bottom_column in [4, 12] {
            let first = LinkCell {
                row: top_row,
                column: 4,
            };
            let second = LinkCell {
                row: bottom_row,
                column: bottom_column,
            };
            let mut tiles = Vec::new();
            for row in 0..BOARD_ROWS as u8 {
                for column in 0..BOARD_COLUMNS as u8 {
                    if (column == first.column && row < first.row)
                        || (column == second.column && row > second.row)
                    {
                        continue;
                    }
                    let kind = if (LinkCell { row, column }) == first
                        || (LinkCell { row, column }) == second
                    {
                        tile(9)
                    } else {
                        tile(2)
                    };
                    tiles.push((row, column, kind));
                }
            }
            let mut board = MahjongLinkBoard::fixture(&tiles);
            let path = board.link_path(first, second).unwrap();
            assert!(path.iter().any(|point| point.row == -1));
            assert!(path.iter().any(|point| point.row == BOARD_ROWS as i8));
            assert!(board.link_path(second, first).is_some());
            let remaining = board.remaining();
            assert_eq!(board.remove_pair(first, second).unwrap().0, 9);
            assert_eq!(board.remaining(), remaining - 2);

            if top_row > 0 {
                tiles.push((top_row - 1, first.column, tile(2)));
                let board = MahjongLinkBoard::fixture(&tiles);
                assert!(board.link_path(first, second).is_none());
            }
        }
    }
}
