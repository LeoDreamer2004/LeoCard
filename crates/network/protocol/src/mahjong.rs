use crate::{
    AvatarId, MatchId, PlayerGameProfiles, PlayerId, PlayerReferenceChange, ProfileId, SeatId,
};
use leocard_mahjong::{
    MahjongClaim, MahjongClaimOption, MahjongHandReplacementError, MahjongMatchLength,
    MahjongMatchProgress, MahjongMeldKind, MahjongRuleSet, MahjongScoreResult, MahjongTile,
    MahjongTileKind, MahjongWind,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: MahjongRuleSet,
    pub players: Vec<MahjongPlayerState>,
    pub your_hand: Vec<MahjongTile>,
    pub your_drawn_tile: Option<MahjongTile>,
    pub discards: Vec<MahjongDiscardView>,
    pub dealer: PlayerId,
    pub prevalent_wind: MahjongWind,
    pub sequence_index: u8,
    pub current_player: PlayerId,
    pub wall_len: u16,
    pub match_scores: [i32; 4],
    pub pending_claim: Option<MahjongPendingClaimView>,
    pub can_self_draw: bool,
    pub can_legal_self_draw: bool,
    pub concealed_kong_options: Vec<MahjongTileKind>,
    pub added_kong_options: Vec<MahjongTile>,
    pub phase: MahjongPhaseView,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub seat_wind: MahjongWind,
    pub concealed_count: u8,
    pub revealed_hand: Option<Vec<MahjongTile>>,
    pub melds: Vec<MahjongPublicMeldView>,
    pub flowers: Vec<MahjongTile>,
    pub dead_hand: bool,
    pub auto_play: bool,
    pub ready: bool,
    pub connected: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongPendingClaimView {
    pub source: PlayerId,
    pub tile: MahjongTile,
    pub robbing_kong: bool,
    pub your_options: Vec<MahjongClaimOption>,
    pub can_legal_win: bool,
    pub your_response: Option<MahjongClaim>,
    pub waiting_for: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongDiscardView {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub claimed_by: Option<PlayerId>,
    pub from_drawn: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongPublicMeldView {
    pub kind: MahjongMeldKind,
    pub tile: Option<MahjongTileKind>,
    pub claimed_from: Option<PlayerId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongWinView {
    pub player: PlayerId,
    pub from: Option<PlayerId>,
    pub concealed_hand: bool,
    pub winning_tile: MahjongTile,
    pub score: MahjongScoreResult,
    pub wait_kind_count: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongHandResultView {
    pub winners: Vec<MahjongWinView>,
    pub exhaustive_draw: bool,
    pub match_length: MahjongMatchLength,
    pub match_progress: MahjongMatchProgress,
    pub deltas: [i32; 4],
    pub match_scores: [i32; 4],
    pub match_complete: bool,
    pub sequence_index: u8,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongPhaseView {
    Dealing { batch: u8 },
    ReplacingFlower { player: PlayerId },
    Playing,
    WaitingForClaims,
    Finished { result: MahjongHandResultView },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongViolation {
    InvalidPlayer,
    NotPlayersTurn,
    WrongPhase,
    TileNotInHand,
    InvalidClaim,
    AlreadyResponded,
    CannotWin,
    CannotKong,
    InvalidDeveloperHand(MahjongHandReplacementError),
}

impl fmt::Display for MahjongViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidDeveloperHand(reason) => return reason.fmt(f),
            Self::InvalidPlayer => "玩家身份无效",
            Self::NotPlayersTurn => "还没有轮到你出牌",
            Self::WrongPhase => "当前阶段不能执行这个操作",
            Self::TileNotInHand => "这张牌不在你的手中",
            Self::InvalidClaim => "当前不能这样吃、碰、杠或和",
            Self::AlreadyResponded => "你已经响应过这张牌",
            Self::CannotWin => "当前手牌不能和牌",
            Self::CannotKong => "当前不能开杠",
        })
    }
}
