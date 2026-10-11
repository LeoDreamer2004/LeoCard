use super::super::{GameState, GuessResult, MatchState, reference_point_deltas};
use crate::{AttributeFeedback, Catalogue, EntryKind, MatchQuality, MinecraftGuessRuleSet};

fn result(qualities: &[MatchQuality]) -> GuessResult {
    GuessResult::Known {
        entry: Catalogue::bundled()
            .find(EntryKind::Block, "minecraft:stone")
            .unwrap()
            .clone(),
        matches: qualities
            .iter()
            .map(|quality| AttributeFeedback {
                quality: *quality,
                direction: None,
            })
            .collect(),
        correct: false,
    }
}

#[test]
fn closeness_and_matching_are_independent_and_unknowns_earn_nothing() {
    use MatchQuality::{Exact, Partial, Wrong};
    let different = result(&[Exact, Partial, Wrong]);
    assert_eq!(different.closeness(), Some(3));
    assert_eq!(different.score(None, 0).during_guess, 1);
    assert_eq!(different.score(Some(2), 0).during_guess, 1);
    assert_eq!(different.score(Some(3), 0).during_guess, 0);
    assert_eq!(different.score(Some(4), 0).during_guess, 0);
    assert_eq!(
        result(&[Exact, Partial, Exact])
            .score(Some(6), 0)
            .during_guess,
        1
    );
    assert_eq!(
        result(&[Exact, Exact, Exact])
            .score(Some(6), 0)
            .during_guess,
        2
    );
    let unknown = GuessResult::Unknown {
        name: "不存在的方块".to_owned(),
    };
    assert_eq!(unknown.closeness(), None);
    assert_eq!(unknown.score(None, 0).during_guess, 0);
    assert_eq!(unknown.score(None, 0).after_answer, 0);
}

#[test]
fn speed_counts_skips_and_unknowns_and_first_known_guess_still_improves() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    for (turns, win_points) in [(0, 7), (2, 6), (4, 5)] {
        let mut game = GameState::new(
            answer.clone(),
            vec![0, 1],
            MinecraftGuessRuleSet {
                allow_unknown_guesses: true,
                candidate_hints: false,
                ..MinecraftGuessRuleSet::default()
            },
        );
        for turn in 0..turns {
            let player = (turn % 2) as u8;
            if turn % 2 == 0 {
                game.guess(player, &format!("未知条目{turn}")).unwrap();
            } else {
                game.skip(player).unwrap();
            }
        }
        assert!(game.take_score_gains().is_empty());
        game.guess(0, &answer.id).unwrap();
        assert_eq!(game.take_score_gains(), vec![(0, 3), (0, win_points)]);
        assert!(game.take_score_gains().is_empty());
        assert!(game.guess(0, &answer.id).is_err());
        assert!(game.take_score_gains().is_empty());
    }
}

#[test]
fn points_accumulate_across_games_and_rank_awards_scale_with_rounds() {
    let mut state = MatchState::new(vec![2, 0, 1, 3], 3);
    state.record_gains([(2, 3), (2, 7), (0, 1)]);
    assert!(state.advance());
    state.record_gains([(2, 2), (3, 5)]);
    assert_eq!(
        state.scores().collect::<Vec<_>>(),
        vec![(2, 12), (0, 1), (1, 0), (3, 5)]
    );
    assert_eq!((state.gains()[3].before, state.gains()[3].after), (10, 12));
    assert_eq!(
        reference_point_deltas(&[4, 3, 2, 1], 2),
        Some(vec![2, 1, -1, -2])
    );
    assert_eq!(
        reference_point_deltas(&[4, 3, 2, 1], 3),
        Some(vec![4, 2, -2, -4])
    );
    assert_eq!(
        reference_point_deltas(&[5, 4, 3, 3, 2, 1], 5),
        Some(vec![4, 2, 0, 0, -2, -4])
    );
    assert_eq!(
        reference_point_deltas(&[4, 4, 2, 1], 1),
        Some(vec![2, 2, -1, -2])
    );
}
