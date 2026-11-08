use super::MahjongWinFeedback;
use crate::automation::BotDecision;
use crate::{AutomaticActionDelayState, RoomSession};
use leocard_mahjong::{
    MahjongBotAction, MahjongBotObservation, MahjongGameState, MahjongRuleSet, MahjongTile,
    MahjongWallBreak,
};
use leocard_protocol::{MatchId, PlayerReferenceChange};
use std::time::Duration;

pub(super) const MAHJONG_DEAL_INTERVAL: Duration = Duration::from_millis(320);
// The first roll also runs after the lobby-to-table seat transition.
pub(super) const MAHJONG_DICE_INTERVAL: Duration = Duration::from_millis(2_000);

#[derive(Clone, Debug)]
pub struct MahjongSession {
    pub(super) bot_decision: BotDecision<MahjongBotObservation, MahjongBotAction>,
    pub(super) win_feedback: MahjongWinFeedback,
    pub(super) room: RoomSession,
    pub(super) rules: MahjongRuleSet,
    pub(super) shuffled_deck: Option<Vec<MahjongTile>>,
    pub(super) opening_dice: MahjongWallBreak,
    pub(super) game: Option<MahjongGameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) automatic_action_delay: Option<AutomaticActionDelayState>,
    pub(super) deal_delay: Duration,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}
