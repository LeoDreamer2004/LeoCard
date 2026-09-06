use crate::{AutoPlayDelayState, RoomSession};
use leocard_protocol::{MatchId, PlayerId, PlayerReferenceChange, UnoProfileStats};
use leocard_uno::{GameState, UnoCard, UnoRuleSet};
use std::time::Duration;

pub(super) const DRAW_REVEAL_START_DELAY: Duration = Duration::from_millis(780);
pub(super) const COLOR_ROULETTE_REVEAL_START_DELAY: Duration = Duration::from_millis(1_000);
pub(super) const DRAW_REVEAL_INTERVAL: Duration = Duration::from_millis(180);

#[derive(Clone, Debug)]
pub(super) struct PendingDrawReveal {
    pub(super) player: PlayerId,
    pub(super) cards: Vec<UnoCard>,
    pub(super) revealed: usize,
    pub(super) remaining: Duration,
}

#[derive(Clone, Debug)]
pub struct UnoSession {
    pub(super) room: RoomSession,
    pub(super) rules: UnoRuleSet,
    pub(super) shuffled_deck: Option<Vec<UnoCard>>,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) match_profile_stats: Vec<UnoProfileStats>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) auto_play_delay: Option<AutoPlayDelayState>,
    pub(super) pending_draw_reveal: Option<PendingDrawReveal>,
}
