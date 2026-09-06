use crate::RoomSession;
use leocard_protocol::{
    MatchId, PlayerReferenceChange, ShengjiProfileStats, ShengjiThrowFailureStage,
};
use leocard_shengji::{
    BottomFlipReveal, GameState, ShengjiCard, ShengjiClassifiedPlay, ShengjiPlayerId,
    ShengjiRuleSet, TeamProgress, TrickRecord,
};
use std::time::Duration;

pub(super) const PLAYER_COUNT: u8 = ShengjiRuleSet::PLAYER_COUNT as u8;
pub(super) const DEAL_INTERVAL: Duration = Duration::from_millis(100);
pub(super) const BIDDING_GRACE: Duration = Duration::from_secs(5);
pub(super) const POWER_OUTAGE_BIDDING_GRACE: Duration = Duration::from_secs(10);
pub(super) const BOTTOM_FLIP_START_DELAY: Duration = Duration::from_millis(500);
pub(super) const BOTTOM_FLIP_HOLD_DURATION: Duration = Duration::from_millis(2800);
pub(super) const BOTTOM_COPY_DECISION_TIMEOUT: Duration = Duration::from_secs(10);
pub(super) const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(1);
pub(super) const REDEAL_DELAY: Duration = Duration::from_millis(650);
pub(super) const TRICK_HOLD_DURATION: Duration = Duration::from_millis(1200);
pub(super) const THROW_FAILURE_SHOW_DURATION: Duration = Duration::from_millis(1200);
pub(super) const THROW_FAILURE_RETURN_DURATION: Duration = Duration::from_millis(420);

#[derive(Clone, Debug)]
pub(super) struct HeldThrowFailure {
    pub(super) player: ShengjiPlayerId,
    pub(super) attempted: Vec<ShengjiCard>,
    pub(super) forced: ShengjiClassifiedPlay,
    pub(super) penalty_points: u16,
    pub(super) stage: ShengjiThrowFailureStage,
    pub(super) remaining: Duration,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HandFlowState {
    pub(super) deal_elapsed: Duration,
    pub(super) bidding_remaining: Option<Duration>,
    pub(super) bid_pass_confirmed: [bool; ShengjiRuleSet::PLAYER_COUNT],
    pub(super) bottom_flip_reveal: Option<BottomFlipReveal>,
    pub(super) bottom_flip_remaining: Option<Duration>,
    pub(super) bottom_copy_remaining: Option<Duration>,
    pub(super) automatic_action: Option<(ShengjiPlayerId, Duration)>,
    pub(super) redeal_remaining: Option<Duration>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HeldGamePresentation {
    pub(super) throw_penalties: [u16; ShengjiRuleSet::PLAYER_COUNT],
    pub(super) throw_failure: Option<HeldThrowFailure>,
    pub(super) trick: Option<(TrickRecord, Duration)>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HandStatistics {
    pub(super) profiles: Vec<ShengjiProfileStats>,
    pub(super) finished_settlement_id: Option<MatchId>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}

/// 四人双升的房主权威会话。发牌、亮主窗口和机器人行动都由房主时钟推进。
#[derive(Clone, Debug)]
pub struct ShengjiSession {
    pub(super) room: RoomSession,
    pub(super) rules: ShengjiRuleSet,
    pub(super) shuffled_deck: Option<Vec<ShengjiCard>>,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) hand_number: u32,
    pub(super) teams: TeamProgress,
    pub(super) next_dealer: Option<ShengjiPlayerId>,
    pub(super) flow: HandFlowState,
    pub(super) presentation: HeldGamePresentation,
    pub(super) statistics: HandStatistics,
}
