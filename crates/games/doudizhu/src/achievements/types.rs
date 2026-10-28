use crate::{Mode, PlayKind};

/// Authority-produced, recipient-specific facts. Opening-hand details are released only at settlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DoudizhuAchievementFacts {
    Play(DoudizhuPlayFacts),
    FlowerBomb,
    Hand(DoudizhuHandFacts),
    Match(DoudizhuMatchFacts),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoudizhuPlayFacts {
    pub mode: Mode,
    pub kind: PlayKind,
    pub card_count: u8,
    pub rank: u8,
    pub is_lead: bool,
    pub beats_airplane: bool,
    pub trick_bombs: u32,
    pub consecutive_bombs: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoudizhuHandFacts {
    pub landlord: bool,
    pub won: bool,
    pub spring: bool,
    pub finished_cards: bool,
    pub finishing_bomb: bool,
    pub unavailable_exchange: bool,
    pub all_bomb_hand: bool,
    pub play_count: u32,
    pub score: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoudizhuMatchFacts {
    pub score: i32,
    pub perfect_spring: bool,
    pub first_to_last: bool,
}
