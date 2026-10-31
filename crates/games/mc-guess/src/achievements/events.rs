use crate::{McGuessActionStatistics, McGuessHandStatistics};
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub enum McGuessAchievementEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<McGuessActionStatistics>,
    },
    HandAnalyzed {
        player: PlayerId,
        statistics: McGuessHandStatistics,
    },
}

impl From<McGuessAchievementEvent> for GameFacts {
    fn from(event: McGuessAchievementEvent) -> Self {
        Self::new(event)
    }
}
