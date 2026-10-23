use super::{GuessHandStatistics, GuessPlayStatistics};
use crate::{GuessMode, MatchProgress};

#[derive(Clone, Debug)]
pub(crate) struct MatchAnalysis {
    players: Vec<PlayerMatchProgress>,
    all_solved: bool,
}

#[derive(Clone, Debug)]
struct PlayerMatchProgress {
    player: u8,
    round_wins: u32,
    total_wins: u32,
}

impl MatchAnalysis {
    pub fn new(players: &[u8]) -> Self {
        Self {
            players: players
                .iter()
                .map(|player| PlayerMatchProgress {
                    player: *player,
                    round_wins: 0,
                    total_wins: 0,
                })
                .collect(),
            all_solved: true,
        }
    }

    pub fn finish(
        &mut self,
        progress: MatchProgress,
        winner: Option<u8>,
        consistency: impl Iterator<Item = (u8, bool)>,
        scores: &[(u8, u32)],
    ) -> Vec<GuessHandStatistics> {
        self.all_solved &= winner.is_some();
        if let Some(winner) = winner {
            let state = self
                .players
                .iter_mut()
                .find(|state| state.player == winner)
                .expect("match winner");
            state.round_wins += 1;
            state.total_wins += 1;
        }
        let round_end = progress.mode() == GuessMode::TurnBased
            && progress.game_in_round() == progress.player_count;
        let highest = scores
            .iter()
            .map(|(_, score)| *score)
            .max()
            .expect("match players");
        let facts = consistency
            .map(|(player, consistent)| {
                let state = self
                    .players
                    .iter()
                    .find(|state| state.player == player)
                    .expect("match player");
                GuessHandStatistics {
                    player,
                    consistent,
                    round_sweep: progress.player_count >= 3
                        && round_end
                        && state.round_wins == u32::from(progress.player_count),
                    match_sweep: progress.mode() == GuessMode::TurnBased
                        && progress.player_count >= 3
                        && progress.round_count().is_some_and(|rounds| rounds >= 3)
                        && progress.is_last_game()
                        && state.total_wins == u32::from(progress.total_games()),
                    no_answers_top: progress.is_last_game()
                        && self.all_solved
                        && state.total_wins == 0
                        && scores
                            .iter()
                            .any(|(id, score)| *id == player && *score == highest),
                }
            })
            .collect();
        if round_end {
            for state in &mut self.players {
                state.round_wins = 0;
            }
        }
        facts
    }

    pub fn finish_graph(
        &mut self,
        progress: MatchProgress,
        statistics: &[(u8, GuessPlayStatistics)],
        scores: &[(u8, u32)],
        mut facts: Vec<GuessHandStatistics>,
    ) -> Vec<GuessHandStatistics> {
        let nodes = statistics
            .first()
            .map_or(0, |(_, stats)| stats.categories.total().games);
        let solved: u32 = statistics
            .iter()
            .map(|(_, stats)| stats.categories.total().wins)
            .sum();
        self.all_solved &= nodes > 0 && solved == nodes;
        let highest = scores
            .iter()
            .map(|(_, score)| *score)
            .max()
            .expect("match players");
        for &(player, statistics) in statistics {
            let wins = statistics.categories.total().wins;
            let state = self
                .players
                .iter_mut()
                .find(|state| state.player == player)
                .expect("graph player");
            state.total_wins += wins;
            if let Some(fact) = facts.iter_mut().rev().find(|fact| fact.player == player) {
                fact.round_sweep = progress.player_count >= 3 && nodes > 0 && wins == nodes;
                fact.no_answers_top = progress.is_last_game()
                    && self.all_solved
                    && state.total_wins == 0
                    && scores
                        .iter()
                        .any(|(id, score)| *id == player && *score == highest);
            }
        }
        facts
    }
}
