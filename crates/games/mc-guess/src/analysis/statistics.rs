use super::McGuessCategoryStatistics;
use crate::{McGuessEntryKind, McGuessResult};

/// Accepted actions, finalized when a question or graph completes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct McGuessPlayStatistics {
    pub categories: McGuessCategoryStatistics,
    pub known_guesses: u32,
    pub non_worse_guesses: u32,
    pub unknown_guesses: u32,
    pub unknown_enabled_guesses: u32,
    pub skips: u32,
}

impl McGuessPlayStatistics {
    pub(crate) fn record_guess(&mut self, result: &McGuessResult, previous: Option<u32>) -> bool {
        let known = matches!(result, McGuessResult::Known { .. });
        let non_worse = result
            .closeness()
            .is_some_and(|score| previous.is_none_or(|before| score >= before));
        self.known_guesses = self.known_guesses.saturating_add(u32::from(known));
        self.non_worse_guesses = self.non_worse_guesses.saturating_add(u32::from(non_worse));
        self.unknown_guesses = self.unknown_guesses.saturating_add(u32::from(!known));
        non_worse
    }

    pub(crate) fn finish_game(&mut self, kind: McGuessEntryKind, won: bool, allow_unknown: bool) {
        self.categories.record(kind, won);
        if allow_unknown {
            self.unknown_enabled_guesses = self.known_guesses.saturating_add(self.unknown_guesses);
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.categories.merge(other.categories);
        for (total, count) in [
            (&mut self.known_guesses, other.known_guesses),
            (&mut self.non_worse_guesses, other.non_worse_guesses),
            (&mut self.unknown_guesses, other.unknown_guesses),
            (
                &mut self.unknown_enabled_guesses,
                other.unknown_enabled_guesses,
            ),
            (&mut self.skips, other.skips),
        ] {
            *total = total.saturating_add(count);
        }
    }
}
