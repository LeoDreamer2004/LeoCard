use crate::analysis::{GuessAnalysisInput, MatchAnalysis, RoundAnalysis};
use crate::{
    McGuessAttributeFeedback, McGuessCatalogue, McGuessEntryKind, McGuessHandStatistics,
    McGuessMatchLength, McGuessMatchProgress, McGuessMatchQuality, McGuessMode,
    McGuessPlayStatistics, McGuessResult,
};
use leocard_game_common::PlayerId;

fn known(quality: McGuessMatchQuality, correct: bool) -> McGuessResult {
    McGuessResult::Known {
        entry: McGuessCatalogue::bundled()
            .find(McGuessEntryKind::Block, "minecraft:stone")
            .unwrap()
            .clone(),
        matches: vec![McGuessAttributeFeedback {
            quality,
            direction: None,
        }],
        correct,
    }
}

fn guess(
    analysis: &mut RoundAnalysis,
    result: &McGuessResult,
    previous: Option<u32>,
    turns: usize,
) {
    analysis.guess(GuessAnalysisInput {
        player: PlayerId(0),
        mode: McGuessMode::TurnBased,
        result,
        previous,
        turns,
        guess_count: turns,
        character_count: None,
        skipped: &[],
    });
}

#[test]
fn action_runs_and_consistency_follow_accepted_guesses() {
    let mut analysis = RoundAnalysis::new(&[PlayerId(0), PlayerId(1), PlayerId(2)]);
    let unknown = McGuessResult::Unknown {
        name: "未知".into(),
    };
    for turn in 0..3 {
        guess(&mut analysis, &unknown, None, turn);
    }
    assert_eq!(analysis.take_actions().last().unwrap().unknown_run, 3);
    analysis.skip(PlayerId(0));
    guess(&mut analysis, &unknown, None, 4);
    assert_eq!(analysis.take_actions().last().unwrap().unknown_run, 1);
    for _ in 0..3 {
        analysis.skip(PlayerId(0));
    }
    assert_eq!(analysis.take_actions().last().unwrap().skip_run, 3);
    let exact = known(McGuessMatchQuality::Exact, false);
    for turn in 5..10 {
        guess(&mut analysis, &exact, Some(2), turn);
    }
    let correct = known(McGuessMatchQuality::Exact, true);
    guess(&mut analysis, &correct, Some(2), 11);
    let win = analysis.take_actions().pop().unwrap();
    assert!(win.ended_matching_run && win.won_after_skip && win.long_search_win);
    assert!(
        !analysis
            .consistency()
            .find(|(player, _)| *player == PlayerId(0))
            .unwrap()
            .1
    );

    let mut analysis = RoundAnalysis::new(&[PlayerId(0), PlayerId(1), PlayerId(2)]);
    guess(&mut analysis, &exact, Some(2), 0);
    assert!(analysis.consistency().next().unwrap().1);
    guess(
        &mut analysis,
        &known(McGuessMatchQuality::Partial, false),
        Some(2),
        3,
    );
    assert!(!analysis.consistency().next().unwrap().1);
}

#[test]
fn sweeps_require_full_rounds_and_a_complete_three_round_match() {
    let mut analysis = MatchAnalysis::new(&[PlayerId(0), PlayerId(1), PlayerId(2)]);
    for index in 0..9 {
        let facts = analysis.finish(
            McGuessMatchProgress {
                game_index: index,
                player_count: 3,
                length: McGuessMatchLength::Rounds(3),
            },
            Some(PlayerId(0)),
            [
                (PlayerId(0), true),
                (PlayerId(1), false),
                (PlayerId(2), false),
            ]
            .into_iter(),
            &[(PlayerId(0), 50), (PlayerId(1), 0), (PlayerId(2), 0)],
        );
        assert_eq!(facts[0].round_sweep, index % 3 == 2);
        assert_eq!(facts[0].match_sweep, index == 8);
    }
}

#[test]
fn no_answer_victory_requires_every_game_to_be_solved() {
    for unsolved in [false, true] {
        let mut analysis = MatchAnalysis::new(&[PlayerId(0), PlayerId(1), PlayerId(2)]);
        for index in 0..3 {
            let facts = analysis.finish(
                McGuessMatchProgress {
                    game_index: index,
                    player_count: 3,
                    length: McGuessMatchLength::Rounds(1),
                },
                if unsolved && index == 0 {
                    None
                } else {
                    Some(PlayerId(1))
                },
                [
                    (PlayerId(0), true),
                    (PlayerId(1), true),
                    (PlayerId(2), false),
                ]
                .into_iter(),
                &[(PlayerId(0), 20), (PlayerId(1), 19), (PlayerId(2), 0)],
            );
            assert_eq!(facts[0].no_answers_top, index == 2 && !unsolved);
        }
    }
}

#[test]
fn graph_match_facts_require_real_node_wins_and_keep_previous_graphs() {
    for (previous_wins, expected) in [([0, 3, 0], true), ([1, 2, 0], false), ([0, 2, 0], false)] {
        let mut analysis = MatchAnalysis::new(&[PlayerId(0), PlayerId(1), PlayerId(2)]);
        for index in 0..2 {
            let wins = if index == 0 { previous_wins } else { [0, 0, 3] };
            let statistics: Vec<_> = wins
                .into_iter()
                .enumerate()
                .map(|(player, wins)| {
                    let mut stats = McGuessPlayStatistics::default();
                    stats.categories.block.games = 3;
                    stats.categories.block.wins = wins;
                    (PlayerId::from_index(player), stats)
                })
                .collect();
            let completed = (0..3)
                .map(|player| McGuessHandStatistics {
                    player: PlayerId(player),
                    consistent: true,
                    round_sweep: false,
                    match_sweep: false,
                    no_answers_top: false,
                })
                .collect();
            let facts = analysis.finish_graph(
                McGuessMatchProgress {
                    game_index: index,
                    player_count: 3,
                    length: McGuessMatchLength::Graph(2),
                },
                &statistics,
                &[(PlayerId(0), 100), (PlayerId(1), 90), (PlayerId(2), 80)],
                completed,
            );
            assert_eq!(facts.len(), 3);
            assert!(
                facts
                    .iter()
                    .all(|fact| fact.consistent && !fact.match_sweep)
            );
            assert_eq!(facts[0].no_answers_top, index == 1 && expected);
            if index == 1 {
                assert!(facts[2].round_sweep);
            } else {
                assert_eq!(facts[1].round_sweep, previous_wins[1] == 3);
            }
        }
    }
}
