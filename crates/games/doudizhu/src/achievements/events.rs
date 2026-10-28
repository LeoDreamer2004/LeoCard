use super::DoudizhuAchievementFacts;
use leocard_achievements::{GameFacts, ParticipantId};

#[derive(Clone, Debug)]
pub enum DoudizhuAchievementEvent {
    ActionAnalyzed {
        player: ParticipantId,
        statistics: Box<DoudizhuAchievementFacts>,
    },
}

impl From<DoudizhuAchievementEvent> for GameFacts {
    fn from(event: DoudizhuAchievementEvent) -> Self {
        Self::new(event)
    }
}
