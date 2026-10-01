use super::MahjongWinFeedback;
use crate::{AutoPlayDelayState, RoomSession};
use leocard_mahjong::{GameState, MahjongRuleSet, MahjongTile};
use leocard_protocol::{MatchId, PlayerReferenceChange};
use std::time::Duration;

pub(super) const MAHJONG_DEAL_INTERVAL: Duration = Duration::from_millis(320);

#[derive(Clone, Debug)]
pub struct MahjongSession {
    pub(super) win_feedback: MahjongWinFeedback,
    pub(super) room: RoomSession,
    pub(super) rules: MahjongRuleSet,
    pub(super) shuffled_deck: Option<Vec<MahjongTile>>,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) auto_play_delay: Option<AutoPlayDelayState>,
    pub(super) deal_delay: Duration,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}
