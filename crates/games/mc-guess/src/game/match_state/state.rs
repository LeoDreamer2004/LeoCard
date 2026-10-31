use super::{McGuessMatchLength, McGuessMatchProgress};
use crate::analysis::MatchAnalysis;
use crate::game::{McGuessGameState, McGuessScoreGain};
use crate::{McGuessHandStatistics, McGuessMode, McGuessPlayStatistics, McGuessRuleSet};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub struct McGuessMatchState {
    players: Vec<PlayerId>,
    progress: McGuessMatchProgress,
    scores: Vec<u32>,
    gains: Vec<McGuessScoreGain>,
    statistics: Vec<McGuessPlayStatistics>,
    analysis: MatchAnalysis,
}

impl McGuessMatchState {
    pub fn new(players: Vec<PlayerId>, rules: McGuessRuleSet) -> Self {
        assert!(!players.is_empty());
        assert!(rules.is_valid());
        Self {
            progress: McGuessMatchProgress {
                game_index: 0,
                length: match rules.mode {
                    McGuessMode::TurnBased => McGuessMatchLength::Rounds(rules.turn_based.rounds),
                    McGuessMode::Race => McGuessMatchLength::Games(rules.race.games),
                    McGuessMode::Graph => McGuessMatchLength::Graph(if rules.graph.round_robin {
                        players.len() as u8
                    } else {
                        1
                    }),
                },
                player_count: players.len() as u8,
            },
            scores: vec![0; players.len()],
            gains: Vec::new(),
            statistics: vec![McGuessPlayStatistics::default(); players.len()],
            analysis: MatchAnalysis::new(&players),
            players,
        }
    }

    pub fn progress(&self) -> McGuessMatchProgress {
        self.progress
    }

    pub fn turn_order(&self) -> Vec<PlayerId> {
        let mut players = self.players.clone();
        if matches!(
            self.progress.mode(),
            McGuessMode::TurnBased | McGuessMode::Graph
        ) {
            players.rotate_left(usize::from(self.progress.game_in_round() - 1));
        }
        players
    }

    pub fn record_gains(&mut self, gains: impl IntoIterator<Item = (PlayerId, u32)>) {
        for (player, points) in gains {
            let index = self
                .players
                .iter()
                .position(|id| *id == player)
                .expect("match participant");
            let before = self.scores[index];
            let after = before.saturating_add(points);
            self.scores[index] = after;
            self.gains.push(McGuessScoreGain {
                player,
                before,
                after,
            });
        }
    }

    pub fn scores(&self) -> impl Iterator<Item = (PlayerId, u32)> + '_ {
        self.players
            .iter()
            .copied()
            .zip(self.scores.iter().copied())
    }

    pub fn gains(&self) -> &[McGuessScoreGain] {
        &self.gains
    }

    pub fn statistics(&self) -> impl Iterator<Item = (PlayerId, &McGuessPlayStatistics)> + '_ {
        self.players.iter().copied().zip(&self.statistics)
    }

    pub fn finish_round(&mut self, game: &mut McGuessGameState) -> Vec<McGuessHandStatistics> {
        assert!(game.finished());
        let statistics: Vec<_> = game.statistics().collect();
        for &(player, statistics) in &statistics {
            let index = self
                .players
                .iter()
                .position(|id| *id == player)
                .expect("match participant");
            self.statistics[index].merge(statistics);
        }
        let scores: Vec<_> = self.scores().collect();
        if self.progress.mode() == McGuessMode::Graph {
            return self.analysis.finish_graph(
                self.progress,
                &statistics,
                &scores,
                game.take_hand_statistics(),
            );
        }
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
