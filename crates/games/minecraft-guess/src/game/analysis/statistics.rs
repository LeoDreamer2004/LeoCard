use crate::EntryKind;

/// Accepted guesses and skips accumulated only from completed small games.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GuessPlayStatistics {
    pub block_games: u32,
    pub block_wins: u32,
    pub item_games: u32,
    pub item_wins: u32,
    pub known_guesses: u32,
    pub non_worse_guesses: u32,
    pub unknown_guesses: u32,
    pub unknown_enabled_guesses: u32,
    pub skips: u32,
}

impl GuessPlayStatistics {
    pub(in crate::game) fn finish_game(&mut self, kind: EntryKind, won: bool, allow_unknown: bool) {
        let (games, wins) = match kind {
            EntryKind::Block => (&mut self.block_games, &mut self.block_wins),
            EntryKind::Item => (&mut self.item_games, &mut self.item_wins),
        };
        *games = 1;
        *wins = u32::from(won);
        if allow_unknown {
            self.unknown_enabled_guesses = self.known_guesses.saturating_add(self.unknown_guesses);
        }
    }

    pub(in crate::game) fn merge(&mut self, other: Self) {
        for (total, count) in [
            (&mut self.block_games, other.block_games),
            (&mut self.block_wins, other.block_wins),
            (&mut self.item_games, other.item_games),
            (&mut self.item_wins, other.item_wins),
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
