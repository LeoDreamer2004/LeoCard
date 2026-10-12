use super::{
    GameState, GuessHandStatistics, GuessPlayStatistics, ScoreGain, analysis::MatchAnalysis,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchProgress {
    pub game_index: u8,
    pub player_count: u8,
    pub rounds: u8,
}

impl MatchProgress {
    pub fn current_round(self) -> u8 {
        self.game_index / self.player_count + 1
    }

    pub fn game_in_round(self) -> u8 {
        self.game_index % self.player_count + 1
    }

    pub fn total_games(self) -> u8 {
        self.player_count * self.rounds
    }

    pub fn is_last_game(self) -> bool {
        self.game_index + 1 == self.total_games()
    }
}

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
    pub fn new(players: Vec<u8>, rounds: u8) -> Self {
        assert!(!players.is_empty());
        assert!((1..=5).contains(&rounds));
        Self {
            progress: MatchProgress {
                game_index: 0,
                player_count: players.len() as u8,
                rounds,
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
        players.rotate_left(usize::from(self.progress.game_in_round() - 1));
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
