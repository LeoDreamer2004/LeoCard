use crate::PersonalEvent;
use leocard_protocol::{AchievementCounts, ChatMessage, GameEvent, PlayerId, PlayerInteraction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AchievementTier {
    Bronze,
    Silver,
    Gold,
}
impl AchievementTier {
    pub const fn medal_index(self) -> usize {
        match self {
            Self::Gold => 0,
            Self::Silver => 1,
            Self::Bronze => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AchievementCategory {
    #[default]
    Mahjong,
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
    Personal,
}

/// Live facts emitted by game, social, or local systems. Snapshots are never triggers.
#[derive(Clone, Debug)]
pub enum AchievementTrigger {
    Game {
        player: PlayerId,
        event: GameEvent,
    },
    Interaction {
        player: PlayerId,
        event: PlayerInteraction,
    },
    Chat {
        player: PlayerId,
        message: ChatMessage,
    },
    SessionStarted {
        player: PlayerId,
        host: PlayerId,
    },
    Personal(PersonalEvent),
    TrophyTotals(AchievementCounts),
    Signal(&'static str),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AchievementScope {
    #[default]
    Lifetime,
    Match,
    Hand,
}

#[derive(Clone, Copy)]
pub struct AchievementCriterion {
    pub id: &'static str,
    pub amount: fn(&AchievementTrigger) -> u64,
    pub target: u64,
    pub scope: AchievementScope,
}

#[derive(Clone, Copy)]
pub struct AchievementDefinition {
    pub id: &'static str,
    pub category: AchievementCategory,
    pub title: &'static str,
    pub tier: AchievementTier,
    pub description: &'static str,
    pub criteria: &'static [AchievementCriterion],
    /// All groups must be satisfied; any criterion within a group suffices.
    pub requirements: &'static [&'static [&'static str]],
}
