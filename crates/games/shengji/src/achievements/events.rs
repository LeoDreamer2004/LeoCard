use crate::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiClassifiedPlay, ShengjiHandStatistics,
    ShengjiMatchStatistics, ShengjiRedealReason,
};
use leocard_achievements::{GameFacts, ParticipantId};

#[derive(Clone, Debug)]
pub struct ShengjiDeclarationFacts {
    pub player: ParticipantId,
    pub trump: ShengjiBidTrump,
    pub kind: ShengjiBidKind,
}

#[derive(Clone, Debug)]
pub struct ShengjiPlayFacts {
    pub player: ParticipantId,
    pub play: ShengjiClassifiedPlay,
}

#[derive(Clone, Debug)]
pub enum ShengjiAchievementEvent {
    DeclarationChanged {
        declaration: ShengjiDeclarationFacts,
    },
    BottomCopied {
        declaration: ShengjiDeclarationFacts,
    },
    CardsPlayed {
        play: ShengjiPlayFacts,
        is_lead: bool,
    },
    HandAnalyzed {
        player: ParticipantId,
        statistics: Box<ShengjiHandStatistics>,
        match_statistics: ShengjiMatchStatistics,
    },
    RedealRequired {
        reason: ShengjiRedealReason,
    },
    FirstTrickDealerCut {
        player: ParticipantId,
    },
}

impl From<ShengjiAchievementEvent> for GameFacts {
    fn from(event: ShengjiAchievementEvent) -> Self {
        Self::new(event)
    }
}
