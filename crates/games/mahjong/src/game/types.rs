use crate::{
    MahjongMatchLength, MahjongMeld, MahjongMeldKind, MahjongRuleSet, MahjongScoreResult,
    MahjongTile, MahjongTileKind,
};
use leocard_game_common::PlayerId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongDrawOrigin {
    Normal,
    KongReplacement,
    FlowerReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongPlayerState {
    pub(super) id: PlayerId,
    pub(super) hand: Vec<MahjongTile>,
    pub(super) melds: Vec<MahjongMeld>,
    pub(super) flowers: Vec<MahjongTile>,
    pub(super) dead_hand: bool,
    pub(super) hand_revealed: bool,
}

impl MahjongPlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[MahjongTile] {
        &self.hand
    }

    pub fn melds(&self) -> &[MahjongMeld] {
        &self.melds
    }

    pub fn flowers(&self) -> &[MahjongTile] {
        &self.flowers
    }

    pub fn is_concealed_hand(&self) -> bool {
        self.melds.iter().all(|meld| !meld.is_open())
    }

    pub const fn is_dead_hand(&self) -> bool {
        self.dead_hand
    }

    pub const fn is_hand_revealed(&self) -> bool {
        self.hand_revealed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongPublicPlayerState {
    pub id: PlayerId,
    pub concealed_count: usize,
    pub revealed_hand: Option<Vec<MahjongTile>>,
    pub melds: Vec<MahjongPublicMeld>,
    pub flowers: Vec<MahjongTile>,
    pub dead_hand: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongPublicMeld {
    pub kind: MahjongMeldKind,
    /// 他人的暗杠在本盘结算前只公开为四张牌背。
    pub tile: Option<MahjongTileKind>,
    pub claimed_from: Option<PlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongDiscard {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub claimed_by: Option<PlayerId>,
    /// 只公开摸打与手打，不公开手牌中的具体位置。
    pub from_drawn: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongClaimOption {
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongClaim {
    Pass,
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

impl MahjongClaim {
    pub(super) const fn option(self) -> Option<MahjongClaimOption> {
        match self {
            Self::Pass => None,
            Self::Chow { start } => Some(MahjongClaimOption::Chow { start }),
            Self::Pung => Some(MahjongClaimOption::Pung),
            Self::Kong => Some(MahjongClaimOption::Kong),
            Self::Win => Some(MahjongClaimOption::Win),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ClaimPriority {
    pub(super) category: u8,
    pub(super) proximity: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(super) enum ClaimTrigger {
    Discard {
        discard_index: usize,
        from: PlayerId,
        tile: MahjongTile,
    },
    AddedKong {
        player: PlayerId,
        tile: MahjongTile,
        meld_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongPendingClaim {
    pub(super) trigger: ClaimTrigger,
    pub(super) options: [Vec<MahjongClaimOption>; MahjongRuleSet::PLAYER_COUNT as usize],
    pub(super) responses: [Option<MahjongClaim>; MahjongRuleSet::PLAYER_COUNT as usize],
}

impl MahjongPendingClaim {
    pub const fn source_player(&self) -> PlayerId {
        match self.trigger {
            ClaimTrigger::Discard { from, .. } => from,
            ClaimTrigger::AddedKong { player, .. } => player,
        }
    }

    pub const fn tile(&self) -> MahjongTile {
        match self.trigger {
            ClaimTrigger::Discard { tile, .. } | ClaimTrigger::AddedKong { tile, .. } => tile,
        }
    }

    pub const fn is_robbing_kong_window(&self) -> bool {
        matches!(self.trigger, ClaimTrigger::AddedKong { .. })
    }

    pub fn options_for(&self, player: PlayerId) -> Option<&[MahjongClaimOption]> {
        self.options.get(player.index()).map(Vec::as_slice)
    }

    pub fn response_from(&self, player: PlayerId) -> Option<MahjongClaim> {
        self.responses.get(player.index()).copied().flatten()
    }

    pub fn waiting_for(&self) -> Vec<PlayerId> {
        (0..usize::from(MahjongRuleSet::PLAYER_COUNT))
            .filter(|index| !self.options[*index].is_empty() && self.responses[*index].is_none())
            .map(PlayerId::from_index)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongWinRecord {
    pub player: PlayerId,
    pub from: Option<PlayerId>,
    /// No exposed chow, pung or kong; concealed kongs preserve this shape.
    pub concealed_hand: bool,
    pub winning_tile: MahjongTile,
    pub score: MahjongScoreResult,
    pub wait_kind_count: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongMatchProgress {
    pub completed_hands: u8,
    pub exhaustive_draws: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongHandResult {
    pub winners: Vec<MahjongWinRecord>,
    pub exhaustive_draw: bool,
    pub match_length: MahjongMatchLength,
    pub match_progress: MahjongMatchProgress,
    pub deltas: [i32; MahjongRuleSet::PLAYER_COUNT as usize],
    pub match_scores: [i32; MahjongRuleSet::PLAYER_COUNT as usize],
    pub match_complete: bool,
    pub sequence_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongPhase {
    Dealing { batch: u8 },
    ReplacingFlower { player: PlayerId },
    Playing,
    WaitingForClaims(MahjongPendingClaim),
    Finished(MahjongHandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongActionOutcome {
    Discarded {
        player: PlayerId,
        tile: MahjongTile,
    },
    ClaimRecorded {
        player: PlayerId,
    },
    Claimed {
        player: PlayerId,
        source: PlayerId,
        tile: MahjongTile,
        claim: MahjongClaim,
    },
    Drew {
        player: PlayerId,
        tile: MahjongTile,
        origin: MahjongDrawOrigin,
    },
    KongDeclared {
        player: PlayerId,
        tile: MahjongTileKind,
        added: bool,
    },
    FalseWin {
        player: PlayerId,
        deltas: [i32; MahjongRuleSet::PLAYER_COUNT as usize],
    },
    HandFinished(MahjongHandResult),
}

/// Shape completion and minimum-fan eligibility are separate gameplay facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MahjongWinAvailability {
    Unavailable,
    InsufficientFan { points: u16 },
    Legal,
}
