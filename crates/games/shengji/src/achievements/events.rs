use crate::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiClassifiedPlay, ShengjiHandStatistics,
    ShengjiMatchStatistics, ShengjiRedealReason,
};
use leocard_achievements::GameFacts;
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub struct ShengjiDeclarationFacts {
    pub player: PlayerId,
    pub trump: ShengjiBidTrump,
    pub kind: ShengjiBidKind,
}

#[derive(Clone, Debug)]
pub struct ShengjiPlayFacts {
    pub player: PlayerId,
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
        player: PlayerId,
        statistics: Box<ShengjiHandStatistics>,
        match_statistics: ShengjiMatchStatistics,
    },
    RedealRequired {
        reason: ShengjiRedealReason,
    },
    FirstTrickDealerCut {
        player: PlayerId,
    },
}

impl From<ShengjiAchievementEvent> for GameFacts {
    fn from(event: ShengjiAchievementEvent) -> Self {
        Self::new(event)
    }
}
