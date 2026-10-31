use crate::{TexasActionStatistics, TexasHandStatistics, TexasSquidAwardStatistics};
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub enum TexasAchievementEvent {
    HandAnalyzed {
        player: PlayerId,
        statistics: Box<TexasHandStatistics>,
    },
    ActionApplied {
        player: PlayerId,
        statistics: TexasActionStatistics,
    },
    SquidAwarded {
        player: PlayerId,
        statistics: TexasSquidAwardStatistics,
    },
}

impl From<TexasAchievementEvent> for GameFacts {
    fn from(event: TexasAchievementEvent) -> Self {
        Self::new(event)
    }
}
