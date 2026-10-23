use crate::analysis::{GuessAnalysisInput, MatchAnalysis, RoundAnalysis};
use crate::{
    AttributeFeedback, Catalogue, EntryKind, GuessHandStatistics, GuessMode, GuessPlayStatistics,
    GuessResult, MatchLength, MatchProgress, MatchQuality,
};

fn known(quality: MatchQuality, correct: bool) -> GuessResult {
    GuessResult::Known {
        entry: Catalogue::bundled()
            .find(EntryKind::Block, "minecraft:stone")
            .unwrap()
            .clone(),
        matches: vec![AttributeFeedback {
            quality,
            direction: None,
        }],
        correct,
    }
}

fn guess(analysis: &mut RoundAnalysis, result: &GuessResult, previous: Option<u32>, turns: usize) {
    analysis.guess(GuessAnalysisInput {
        player: 0,
        mode: GuessMode::TurnBased,
        result,
        previous,
        turns,
        guess_count: turns,
        skipped: &[],
    });
}

#[test]
fn action_runs_and_consistency_follow_accepted_guesses() {
    let mut analysis = RoundAnalysis::new(&[0, 1, 2]);
    let unknown = GuessResult::Unknown {
        name: "未知".into(),
    };
    for turn in 0..3 {
        guess(&mut analysis, &unknown, None, turn);
    }
    assert_eq!(analysis.take_actions().last().unwrap().unknown_run, 3);
    analysis.skip(0);
    guess(&mut analysis, &unknown, None, 4);
    assert_eq!(analysis.take_actions().last().unwrap().unknown_run, 1);
    for _ in 0..3 {
        analysis.skip(0);
    }
    assert_eq!(analysis.take_actions().last().unwrap().skip_run, 3);
    let exact = known(MatchQuality::Exact, false);
    for turn in 5..10 {
        guess(&mut analysis, &exact, Some(2), turn);
    }
    let correct = known(MatchQuality::Exact, true);
    guess(&mut analysis, &correct, Some(2), 11);
    let win = analysis.take_actions().pop().unwrap();
    assert!(win.ended_matching_run && win.won_after_skip && win.long_search_win);
    assert!(
        !analysis
            .consistency()
            .find(|(player, _)| *player == 0)
            .unwrap()
            .1
    );

    let mut analysis = RoundAnalysis::new(&[0, 1, 2]);
    guess(&mut analysis, &exact, Some(2), 0);
    assert!(analysis.consistency().next().unwrap().1);
    guess(
        &mut analysis,
        &known(MatchQuality::Partial, false),
        Some(2),
        3,
    );
    assert!(!analysis.consistency().next().unwrap().1);
}

#[test]
fn sweeps_require_full_rounds_and_a_complete_three_round_match() {
    let mut analysis = MatchAnalysis::new(&[0, 1, 2]);
    for index in 0..9 {
        let facts = analysis.finish(
            MatchProgress {
                game_index: index,
                player_count: 3,
                length: MatchLength::Rounds(3),
            },
            Some(0),
            [(0, true), (1, false), (2, false)].into_iter(),
            &[(0, 50), (1, 0), (2, 0)],
        );
        assert_eq!(facts[0].round_sweep, index % 3 == 2);
        assert_eq!(facts[0].match_sweep, index == 8);
    }
}

#[test]
fn no_answer_victory_requires_every_game_to_be_solved() {
    for unsolved in [false, true] {
        let mut analysis = MatchAnalysis::new(&[0, 1, 2]);
        for index in 0..3 {
            let facts = analysis.finish(
                MatchProgress {
                    game_index: index,
                    player_count: 3,
                    length: MatchLength::Rounds(1),
                },
                if unsolved && index == 0 {
                    None
                } else {
                    Some(1)
                },
                [(0, true), (1, true), (2, false)].into_iter(),
                &[(0, 20), (1, 19), (2, 0)],
            );
            assert_eq!(facts[0].no_answers_top, index == 2 && !unsolved);
        }
    }
}

#[test]
fn graph_match_facts_require_real_node_wins_and_keep_previous_graphs() {
    for (previous_wins, expected) in [([0, 3, 0], true), ([1, 2, 0], false), ([0, 2, 0], false)] {
        let mut analysis = MatchAnalysis::new(&[0, 1, 2]);
        for index in 0..2 {
            let wins = if index == 0 { previous_wins } else { [0, 0, 3] };
            let statistics: Vec<_> = wins
                .into_iter()
                .enumerate()
                .map(|(player, wins)| {
                    let mut stats = GuessPlayStatistics::default();
                    stats.categories.block.games = 3;
                    stats.categories.block.wins = wins;
                    (player as u8, stats)
                })
                .collect();
            let completed = (0..3)
                .map(|player| GuessHandStatistics {
                    player,
                    consistent: true,
                    round_sweep: false,
                    match_sweep: false,
                    no_answers_top: false,
                })
                .collect();
            let facts = analysis.finish_graph(
                MatchProgress {
                    game_index: index,
                    player_count: 3,
                    length: MatchLength::Graph(2),
                },
                &statistics,
                &[(0, 100), (1, 90), (2, 80)],
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
