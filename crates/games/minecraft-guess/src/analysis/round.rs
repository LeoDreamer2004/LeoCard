use super::types::PlayerRoundProgress;
use super::{CorrectGuess, GuessActionStatistics, GuessPlayStatistics};
use crate::{GuessMode, GuessResult, MatchQuality};
use std::mem;

#[derive(Clone, Copy)]
pub(crate) struct GuessAnalysisInput<'a> {
    pub player: u8,
    pub mode: GuessMode,
    pub result: &'a GuessResult,
    pub previous: Option<u32>,
    pub turns: usize,
    pub guess_count: usize,
    pub character_count: Option<usize>,
    pub skipped: &'a [u8],
}

#[derive(Clone, Debug)]
pub(crate) struct RoundAnalysis {
    players: Vec<PlayerRoundProgress>,
    matching_run: u32,
    matching_guesses: u32,
    matching_players: Vec<u8>,
    pending: Vec<GuessActionStatistics>,
}

impl RoundAnalysis {
    pub fn new(players: &[u8]) -> Self {
        Self {
            players: players
                .iter()
                .copied()
                .map(PlayerRoundProgress::new)
                .collect(),
            matching_run: 0,
            matching_guesses: 0,
            matching_players: Vec::new(),
            pending: Vec::new(),
        }
    }

    pub fn guess(&mut self, input: GuessAnalysisInput<'_>) {
        let count = self.players.len();
        let state = self
            .players
            .iter_mut()
            .find(|state| state.player == input.player)
            .expect("round player");
        state.skip_run = 0;
        let correct = input.result.is_correct();
        let known = matches!(input.result, GuessResult::Known { .. });
        state.unknown_run = if known { 0 } else { state.unknown_run + 1 };
        let non_worse = state.statistics.record_guess(input.result, input.previous);
        state.consistent &= non_worse;
        let qualities = match input.result {
            GuessResult::Known { matches, .. } => Some(matches),
            GuessResult::Unknown { .. } => None,
        };
        let all_matching = input.result.all_matching();
        let all_wrong = qualities.is_some_and(|matches| {
            !matches.is_empty()
                && matches
                    .iter()
                    .all(|feedback| feedback.quality == MatchQuality::Wrong)
        });
        let correct_entry = match input.result {
            GuessResult::Known {
                entry,
                correct: true,
                ..
            } => Some(CorrectGuess {
                id: entry.id.clone(),
                kind: entry.kind,
            }),
            _ => None,
        };
        self.pending.push(GuessActionStatistics {
            player: input.player,
            correct_entry,
            wrong_all_matching: !correct && all_matching,
            first_matching_wrong: !correct && all_matching && self.matching_guesses == 0,
            won_after_others_matching: correct
                && self
                    .matching_players
                    .iter()
                    .any(|&player| player != input.player),
            wrong_length: input
                .character_count
                .is_some_and(|count| input.result.name().chars().count() != count),
            unknown: !known,
            unknown_run: state.unknown_run,
            skip_run: 0,
            all_wrong_after_first: input.guess_count > 0 && all_wrong,
            ended_matching_run: correct && self.matching_run >= 5,
            won_after_skip: correct && state.has_skipped,
            first_cycle_win: correct && input.mode != GuessMode::Race && input.turns < count,
            opening_win: correct && input.mode != GuessMode::Graph && input.turns == 0,
            opponents_all_skipped: correct
                && count >= 3
                && self.players.iter().all(|state| {
                    state.player == input.player || input.skipped.contains(&state.player)
                }),
            long_search_win: correct && input.guess_count + 1 >= 12,
            ..Default::default()
        });
        if !correct && all_matching {
            self.matching_guesses += 1;
            let new_player = !self.matching_players.contains(&input.player);
            if new_player {
                self.matching_players.push(input.player);
            }
            let recipients = if self.matching_guesses == 15 {
                self.matching_players.clone()
            } else if self.matching_guesses > 15 && new_player {
                vec![input.player]
            } else {
                Vec::new()
            };
            self.pending
                .extend(recipients.into_iter().map(|player| GuessActionStatistics {
                    player,
                    unlucky_matching: true,
                    ..Default::default()
                }));
        }
        self.matching_run = if !correct && all_matching {
            self.matching_run + 1
        } else {
            0
        };
    }

    pub fn skip(&mut self, player: u8) {
        let state = self
            .players
            .iter_mut()
            .find(|state| state.player == player)
            .expect("round player");
        state.has_skipped = true;
        state.statistics.skips += 1;
        state.skip_run += 1;
        state.unknown_run = 0;
        self.pending.push(GuessActionStatistics {
            player,
            skip_run: state.skip_run,
            ..Default::default()
        });
    }

    pub fn take_actions(&mut self) -> Vec<GuessActionStatistics> {
        mem::take(&mut self.pending)
    }

    pub fn statistics(&self) -> impl Iterator<Item = (u8, GuessPlayStatistics)> + '_ {
        self.players
            .iter()
            .map(|state| (state.player, state.statistics))
    }

    pub fn consistency(&self) -> impl Iterator<Item = (u8, bool)> + '_ {
        self.players.iter().map(|state| {
            (
                state.player,
                state.consistent && state.statistics.known_guesses > 0,
            )
        })
    }
}
