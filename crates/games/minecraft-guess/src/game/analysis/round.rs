use super::{CorrectGuess, GuessActionStatistics, GuessPlayStatistics, types::PlayerRoundProgress};
use crate::{GuessResult, MatchQuality};
use std::mem;

pub(in crate::game) struct GuessAnalysisInput<'a> {
    pub player: u8,
    pub result: &'a GuessResult,
    pub previous: Option<u32>,
    pub turns: usize,
    pub guess_count: usize,
    pub skipped: &'a [u8],
}

#[derive(Clone, Debug)]
pub(in crate::game) struct RoundAnalysis {
    players: Vec<PlayerRoundProgress>,
    matching_run: u32,
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
        let non_worse = input
            .result
            .closeness()
            .is_some_and(|score| input.previous.is_none_or(|previous| score >= previous));
        state.consistent &= non_worse;
        state.statistics.known_guesses += u32::from(known);
        state.statistics.non_worse_guesses += u32::from(non_worse);
        state.statistics.unknown_guesses += u32::from(!known);
        let qualities = match input.result {
            GuessResult::Known { matches, .. } => Some(matches),
            GuessResult::Unknown { .. } => None,
        };
        let all_matching = qualities.is_some_and(|matches| {
            matches
                .iter()
                .all(|feedback| feedback.quality == MatchQuality::Exact)
        });
        let all_wrong = qualities.is_some_and(|matches| {
            matches
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
            unknown: !known,
            unknown_run: state.unknown_run,
            skip_run: 0,
            all_wrong_after_first: input.guess_count > 0 && all_wrong,
            ended_matching_run: correct && self.matching_run >= 5,
            won_after_skip: correct && state.has_skipped,
            first_cycle_win: correct && input.turns < count,
            opening_win: correct && input.turns == 0,
            opponents_all_skipped: correct
                && count >= 3
                && self.players.iter().all(|state| {
                    state.player == input.player || input.skipped.contains(&state.player)
                }),
            long_search_win: correct && input.guess_count + 1 >= 12,
        });
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
            correct_entry: None,
            wrong_all_matching: false,
            unknown: false,
            unknown_run: 0,
            skip_run: state.skip_run,
            all_wrong_after_first: false,
            ended_matching_run: false,
            won_after_skip: false,
            first_cycle_win: false,
            opening_win: false,
            opponents_all_skipped: false,
            long_search_win: false,
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
