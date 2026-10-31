use crate::UnoActionStatistics;
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;
#[derive(Clone, Debug)]
pub enum UnoAchievementEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<UnoActionStatistics>,
    },
}

impl From<UnoAchievementEvent> for GameFacts {
    fn from(event: UnoAchievementEvent) -> Self {
        Self::new(event)
    }
}
