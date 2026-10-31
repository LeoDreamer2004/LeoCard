use super::DoudizhuAchievementFacts;
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub enum DoudizhuAchievementEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<DoudizhuAchievementFacts>,
    },
}

impl From<DoudizhuAchievementEvent> for GameFacts {
    fn from(event: DoudizhuAchievementEvent) -> Self {
        Self::new(event)
    }
}
