use super::{MatchLength, MatchProgress};
use crate::game::analysis::MatchAnalysis;
use crate::game::{GameState, GuessHandStatistics, GuessPlayStatistics, ScoreGain};
use crate::{GuessMode, MinecraftGuessRuleSet};

#[derive(Clone, Debug)]
pub struct MatchState {
    players: Vec<u8>,
    progress: MatchProgress,
    scores: Vec<u32>,
    gains: Vec<ScoreGain>,
    statistics: Vec<GuessPlayStatistics>,
    analysis: MatchAnalysis,
}

impl MatchState {
    pub fn new(players: Vec<u8>, rules: MinecraftGuessRuleSet) -> Self {
        assert!(!players.is_empty());
        assert!(rules.is_valid());
        Self {
            progress: MatchProgress {
                game_index: 0,
                length: match rules.mode {
                    GuessMode::TurnBased => MatchLength::Rounds(rules.turn_based.rounds),
                    GuessMode::Race => MatchLength::Games(rules.race.games),
                    GuessMode::Graph => MatchLength::Graph(if rules.graph.round_robin {
                        players.len() as u8
                    } else {
                        1
                    }),
                },
                player_count: players.len() as u8,
            },
            scores: vec![0; players.len()],
            gains: Vec::new(),
            statistics: vec![GuessPlayStatistics::default(); players.len()],
            analysis: MatchAnalysis::new(&players),
            players,
        }
    }

    pub fn progress(&self) -> MatchProgress {
        self.progress
    }

    pub fn turn_order(&self) -> Vec<u8> {
        let mut players = self.players.clone();
        if matches!(
            self.progress.mode(),
            GuessMode::TurnBased | GuessMode::Graph
        ) {
            players.rotate_left(usize::from(self.progress.game_in_round() - 1));
        }
        players
    }

    pub fn record_gains(&mut self, gains: impl IntoIterator<Item = (u8, u32)>) {
        for (player, points) in gains {
            let index = self
                .players
                .iter()
                .position(|id| *id == player)
                .expect("match participant");
            let before = self.scores[index];
            let after = before.saturating_add(points);
            self.scores[index] = after;
            self.gains.push(ScoreGain {
                player,
                before,
                after,
            });
        }
    }

    pub fn scores(&self) -> impl Iterator<Item = (u8, u32)> + '_ {
        self.players
            .iter()
            .copied()
            .zip(self.scores.iter().copied())
    }

    pub fn gains(&self) -> &[ScoreGain] {
        &self.gains
    }

    pub fn statistics(&self) -> impl Iterator<Item = (u8, &GuessPlayStatistics)> + '_ {
        self.players.iter().copied().zip(&self.statistics)
    }

    pub fn finish_round(&mut self, game: &GameState) -> Vec<GuessHandStatistics> {
        assert!(game.finished());
        for (player, statistics) in game.statistics() {
            let index = self
                .players
                .iter()
                .position(|id| *id == player)
                .expect("match participant");
            self.statistics[index].merge(statistics);
        }
        // Graphs share archive counters, but do not emit single-question achievements.
        if self.progress.mode() == GuessMode::Graph {
            return Vec::new();
        }
        let scores: Vec<_> = self.scores().collect();
        self.analysis.finish(
            self.progress,
            game.view().winner,
            game.consistency(),
            &scores,
        )
    }

    pub fn advance(&mut self) -> bool {
        if self.progress.is_last_game() {
            return false;
        }
        self.progress.game_index += 1;
        true
    }
}
