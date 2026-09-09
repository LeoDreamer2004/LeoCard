use crate::{
    MahjongMeldKind, MahjongPlayerId, MahjongRuleSet, MahjongScoreResult, MahjongTile,
    MahjongTileKind, Meld,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongDrawOrigin {
    Normal,
    KongReplacement,
    FlowerReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerState {
    pub(super) id: MahjongPlayerId,
    pub(super) hand: Vec<MahjongTile>,
    pub(super) melds: Vec<Meld>,
    pub(super) flowers: Vec<MahjongTile>,
    pub(super) dead_hand: bool,
    pub(super) hand_revealed: bool,
}

impl PlayerState {
    pub const fn id(&self) -> MahjongPlayerId {
        self.id
    }

    pub fn hand(&self) -> &[MahjongTile] {
        &self.hand
    }

    pub fn melds(&self) -> &[Meld] {
        &self.melds
    }

    pub fn flowers(&self) -> &[MahjongTile] {
        &self.flowers
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
pub struct PublicPlayerState {
    pub id: MahjongPlayerId,
    pub concealed_count: usize,
    pub revealed_hand: Option<Vec<MahjongTile>>,
    pub melds: Vec<PublicMeld>,
    pub flowers: Vec<MahjongTile>,
    pub dead_hand: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicMeld {
    pub kind: MahjongMeldKind,
    /// 他人的暗杠在本盘结算前只公开为四张牌背。
    pub tile: Option<MahjongTileKind>,
    pub claimed_from: Option<MahjongPlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Discard {
    pub player: MahjongPlayerId,
    pub tile: MahjongTile,
    pub claimed_by: Option<MahjongPlayerId>,
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
        from: MahjongPlayerId,
        tile: MahjongTile,
    },
    AddedKong {
        player: MahjongPlayerId,
        tile: MahjongTile,
        meld_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendingClaim {
    pub(super) trigger: ClaimTrigger,
    pub(super) options: [Vec<MahjongClaimOption>; MahjongRuleSet::PLAYER_COUNT],
    pub(super) responses: [Option<MahjongClaim>; MahjongRuleSet::PLAYER_COUNT],
}

impl PendingClaim {
    pub const fn source_player(&self) -> MahjongPlayerId {
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

    pub fn options_for(&self, player: MahjongPlayerId) -> Option<&[MahjongClaimOption]> {
        self.options.get(player.0).map(Vec::as_slice)
    }

    pub fn response_from(&self, player: MahjongPlayerId) -> Option<MahjongClaim> {
        self.responses.get(player.0).copied().flatten()
    }

    pub fn waiting_for(&self) -> Vec<MahjongPlayerId> {
        (0..MahjongRuleSet::PLAYER_COUNT)
            .filter(|index| !self.options[*index].is_empty() && self.responses[*index].is_none())
            .map(MahjongPlayerId)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinRecord {
    pub player: MahjongPlayerId,
    pub from: Option<MahjongPlayerId>,
    pub winning_tile: MahjongTile,
    pub score: MahjongScoreResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandResult {
    pub winners: Vec<WinRecord>,
    pub exhaustive_draw: bool,
    pub deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub match_scores: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub match_complete: bool,
    pub sequence_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    Dealing { batch: u8 },
    ReplacingFlower { player: MahjongPlayerId },
    Playing,
    WaitingForClaims(PendingClaim),
    Finished(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionOutcome {
    Discarded {
        player: MahjongPlayerId,
        tile: MahjongTile,
    },
    ClaimRecorded {
        player: MahjongPlayerId,
    },
    Claimed {
        player: MahjongPlayerId,
        source: MahjongPlayerId,
        tile: MahjongTile,
        claim: MahjongClaim,
    },
    Drew {
        player: MahjongPlayerId,
        tile: MahjongTile,
        origin: MahjongDrawOrigin,
    },
    KongDeclared {
        player: MahjongPlayerId,
        tile: MahjongTileKind,
        added: bool,
    },
    FalseWin {
        player: MahjongPlayerId,
        deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    },
    HandFinished(HandResult),
}
