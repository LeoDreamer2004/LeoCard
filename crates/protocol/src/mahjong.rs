use leocard_mahjong::{
    MahjongClaim, MahjongClaimOption, MahjongMeldKind, MahjongRuleSet, MahjongScoreResult,
    MahjongTile, MahjongTileKind, MahjongWind,
};
use serde::{Deserialize, Serialize};

use crate::{AvatarId, MatchId, PlayerGameProfiles, PlayerId, ProfileId, SeatId};

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
    pub your_response: Option<MahjongClaim>,
    pub waiting_for: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongDiscardView {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub claimed_by: Option<PlayerId>,
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
    pub winning_tile: MahjongTile,
    pub score: MahjongScoreResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongHandResultView {
    pub winners: Vec<MahjongWinView>,
    pub exhaustive_draw: bool,
    pub deltas: [i32; 4],
    pub match_scores: [i32; 4],
    pub match_complete: bool,
    pub sequence_index: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongPhaseView {
    Dealing { batch: u8 },
    ReplacingFlower { player: PlayerId },
    Playing,
    WaitingForClaims,
    Finished { result: MahjongHandResultView },
}
