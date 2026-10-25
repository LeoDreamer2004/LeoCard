use crate::{MahjongMatchLength, MahjongMatchProgress, MahjongScoreResult};
use leocard_achievements::{GameFacts, ParticipantId};

#[derive(Clone, Debug)]
pub struct MahjongWinFacts {
    pub player: ParticipantId,
    pub from: Option<ParticipantId>,
    pub concealed_hand: bool,
    pub score: MahjongScoreResult,
    pub wait_kind_count: u8,
}

#[derive(Clone, Debug)]
pub struct MahjongHandFacts {
    pub winners: Vec<MahjongWinFacts>,
    pub match_length: MahjongMatchLength,
    pub match_progress: MahjongMatchProgress,
    pub deltas: [i32; 4],
    pub match_scores: [i32; 4],
    pub match_complete: bool,
}

#[derive(Clone, Debug)]
pub enum MahjongAchievementEvent {
    FlowerReplaced {
        player: ParticipantId,
    },
    WinUnavailable {
        player: ParticipantId,
        points_without_flowers: u16,
    },
    FalseWin {
        player: ParticipantId,
    },
    HandFinished {
        result: MahjongHandFacts,
    },
}

impl From<MahjongAchievementEvent> for GameFacts {
    fn from(event: MahjongAchievementEvent) -> Self {
        Self::new(event)
    }
}
