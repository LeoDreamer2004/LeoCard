use crate::QiGuiActionStatistics;
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;
#[derive(Clone, Debug)]
pub enum QiGuiAchievementEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<QiGuiActionStatistics>,
    },
}

impl From<QiGuiAchievementEvent> for GameFacts {
    fn from(event: QiGuiAchievementEvent) -> Self {
        Self::new(event)
    }
}
