use crate::{LobbyPlayer, MatchId, PlayerId, PlayerReferenceChange, PlayerScore};
use leocard_minecraft_guess::{
    GuessActionStatistics, GuessHandStatistics, MatchProgress, MinecraftGuessRuleSet, RoundView,
    ScoreGain,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinecraftGuessSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<LobbyPlayer>,
    pub rules: MinecraftGuessRuleSet,
    pub progress: MatchProgress,
    pub round: RoundView,
    pub scores: Vec<PlayerScore>,
    pub score_gains: Vec<ScoreGain>,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MinecraftGuessEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<GuessActionStatistics>,
    },
    HandAnalyzed {
        player: PlayerId,
        statistics: GuessHandStatistics,
    },
}
