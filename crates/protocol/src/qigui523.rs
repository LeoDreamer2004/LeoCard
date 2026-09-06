use leocard_qigui523::{QiGuiCard, QiGuiPlayKind};
use serde::{Deserialize, Serialize};

use crate::{AvatarId, MatchId, PlayerGameProfiles, PlayerId, ProfileId, SeatId};

/// 面向单个七鬼五二三客户端生成的状态。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523Snapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<PlayerPublicState>,
    pub your_hand: Vec<QiGuiCard>,
    pub draw_pile_len: u16,
    pub starting_card: StartingCardView,
    pub trick: Option<TrickView>,
    pub turn_timer: Option<TurnTimerView>,
    pub phase: GamePhaseView,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnTimerView {
    pub player: PlayerId,
    pub base_seconds: u16,
    pub reserve_seconds: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerPublicState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u16,
    pub score: u32,
    pub ready: bool,
    pub connected: bool,
    /// 由房主执行贪心策略的托管状态；所有客户端都可见。
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StartingCardView {
    pub player: PlayerId,
    pub card: QiGuiCard,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrickView {
    pub leader: PlayerId,
    pub current_player: PlayerId,
    pub winning_player: Option<PlayerId>,
    pub winning_play: Option<PublicPlay>,
    pub records: Vec<PublicPlayRecord>,
    pub table_points: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicPlay {
    pub kind: QiGuiPlayKind,
    pub cards: Vec<QiGuiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PublicPlayRecord {
    Played { player: PlayerId, play: PublicPlay },
    Passed { player: PlayerId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GamePhaseView {
    Playing,
    Finished {
        match_id: MatchId,
        finisher: PlayerId,
        scores: Vec<PlayerScore>,
        remaining_hands: Vec<RevealedHand>,
        reference_changes: Vec<PlayerReferenceChange>,
        captured_hand_points: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RevealedHand {
    pub player: PlayerId,
    pub cards: Vec<QiGuiCard>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerScore {
    pub player: PlayerId,
    pub score: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerReferenceChange {
    pub player: PlayerId,
    pub profile_id: ProfileId,
    pub delta: i16,
}
