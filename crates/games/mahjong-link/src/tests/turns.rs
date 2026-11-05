use super::*;

#[test]
fn one_push_preserves_the_turn_until_matching_or_passing() {
    for count in 2..=MahjongLinkRuleSet::MAX_PLAYERS {
        let mut game = MahjongLinkGameState::new(count, build_deck()).unwrap();
        let push = MahjongLinkAction::Push {
            row: 0,
            from: PushSide::Right,
            distance: 1,
        };
        assert_eq!(game.act(0, push), Err(MahjongLinkError::PushOutOfBounds));
        assert!(!game.push_used());
        game.act(
            0,
            MahjongLinkAction::Match {
                first: LinkCell { row: 0, column: 0 },
                second: LinkCell { row: 0, column: 1 },
            },
        )
        .unwrap();
        game.act(1, push).unwrap();
        assert_eq!(game.current_player(), Some(1));
        assert_eq!(game.turn(), 1);
        assert!(game.push_used());
        assert_eq!(game.scores()[1], 0);
        let board = game.board().clone();
        assert_eq!(game.act(1, push), Err(MahjongLinkError::PushAlreadyUsed));
        assert_eq!(game.board(), &board);
        assert_eq!(
            game.act(0, MahjongLinkAction::Pass),
            Err(MahjongLinkError::NotPlayersTurn)
        );
        assert_eq!(
            game.act(
                1,
                MahjongLinkAction::Match {
                    first: LinkCell { row: 0, column: 1 },
                    second: LinkCell { row: 0, column: 3 },
                }
            ),
            Err(MahjongLinkError::TilesDoNotMatch)
        );
        assert!(game.push_used());
        assert_eq!(
            game.act(
                1,
                MahjongLinkAction::Match {
                    first: LinkCell { row: 0, column: 1 },
                    second: LinkCell { row: 0, column: 2 },
                }
            )
            .unwrap()
            .points,
            1
        );
        assert_eq!(game.scores()[1], 1);
        assert_eq!(game.turn(), 2);
        assert!(!game.push_used());
        let next = 2 % count;
        assert_eq!(game.current_player(), Some(next));
        game.act(next, push).unwrap();
        let automatic = game.automatic_action();
        assert!(!matches!(automatic, MahjongLinkAction::Push { .. }));
        game.act(next, MahjongLinkAction::Pass).unwrap();
        assert_eq!(game.turn(), 3);
        assert!(!game.push_used());
        assert_eq!(game.consecutive_passes(), 1);
        assert!(!game.complete());
    }
}
