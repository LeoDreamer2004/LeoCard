use super::super::{GameState, GuessResult, MatchState, reference_point_deltas};
use crate::{
    AttributeFeedback, Catalogue, EntryKind, GuessOptions, MatchQuality, MinecraftGuessRuleSet,
    TurnBasedRules,
};

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
    assert_eq!(different.score(None, 0, false).during_guess, 1);
    assert_eq!(different.score(Some(2), 0, false).during_guess, 1);
    assert_eq!(different.score(Some(3), 0, false).during_guess, 0);
    assert_eq!(different.score(Some(4), 0, false).during_guess, 0);
    assert_eq!(
        result(&[Exact, Partial, Exact])
            .score(Some(6), 0, false)
            .during_guess,
        1
    );
    assert_eq!(
        result(&[Exact, Exact, Exact])
            .score(Some(6), 0, false)
            .during_guess,
        2
    );
    let unknown = GuessResult::Unknown {
        name: "不存在的方块".to_owned(),
    };
    assert_eq!(unknown.closeness(), None);
    assert_eq!(unknown.score(None, 0, false).during_guess, 0);
    assert_eq!(unknown.score(None, 0, false).after_answer, 0);
}

#[test]
fn first_all_green_guess_gets_bonus_even_when_it_is_not_the_answer() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:blue_wool")
        .unwrap()
        .clone();
    let mut game = GameState::new(answer, vec![0, 1], MinecraftGuessRuleSet::default());
    game.guess(0, "minecraft:red_wool").unwrap();
    assert!(game.view().guesses[0].result.all_matching());
    assert!(!game.finished());
    assert_eq!(game.take_score_gains(), vec![(0, 6)]);

    assert!(game.guess(1, "minecraft:red_wool").is_err());
    assert!(game.take_score_gains().is_empty());
    game.guess(1, "minecraft:green_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(1, 2)]);
    game.guess(0, "minecraft:blue_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(0, 6)]);
}

#[test]
fn correct_guesses_only_earn_answer_and_speed_points() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    for (turns, win_points) in [(0, 7), (2, 6), (4, 5)] {
        let mut game = GameState::new(
            answer.clone(),
            vec![0, 1],
            MinecraftGuessRuleSet {
                turn_based: TurnBasedRules {
                    options: GuessOptions {
                        allow_unknown_guesses: true,
                        candidate_hints: false,
                        ..MinecraftGuessRuleSet::default().turn_based.options
                    },
                    ..MinecraftGuessRuleSet::default().turn_based
                },
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
        assert_eq!(game.take_score_gains(), vec![(0, win_points)]);
        assert!(game.take_score_gains().is_empty());
        assert!(game.guess(0, &answer.id).is_err());
        assert!(game.take_score_gains().is_empty());
    }
}

#[test]
fn points_accumulate_across_games_and_rank_awards_scale_with_rounds() {
    let mut state = MatchState::new(
        vec![2, 0, 1, 3],
        MinecraftGuessRuleSet {
            turn_based: TurnBasedRules {
                rounds: 3,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    state.record_gains([(2, 3), (2, 7), (0, 1)]);
    assert!(state.advance());
    state.record_gains([(2, 2), (3, 5)]);
    assert_eq!(
        state.scores().collect::<Vec<_>>(),
        vec![(2, 12), (0, 1), (1, 0), (3, 5)]
    );
    assert_eq!((state.gains()[3].before, state.gains()[3].after), (10, 12));
    let awards: &[&[i16]] = &[
        &[2, -2],
        &[2, 0, -2],
        &[3, -2, -2, -3],
        &[3, 2, 0, -2, -3],
        &[5, 3, 1, -1, -3, -5],
        &[5, 3, 1, 0, -1, -3, -5],
        &[5, 3, 2, 1, -1, -2, -3, -5],
    ];
    for &tiers in awards {
        let scores: Vec<_> = (1..=tiers.len() as u32).rev().collect();
        for rounds in 1..=5 {
            let multiplier = if rounds >= 3 { 2 } else { 1 };
            let expected: Vec<_> = tiers.iter().map(|tier| tier * multiplier).collect();
            assert_eq!(reference_point_deltas(&scores, rounds), Some(expected));
        }
    }
    assert_eq!(
        reference_point_deltas(&[4, 4, 2, 1], 1),
        Some(vec![3, 3, -2, -3])
    );
    assert_eq!(reference_point_deltas(&[1; 9], 1), None);
}
