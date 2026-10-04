use crate::{UnoCard, UnoColor, UnoPlayerId, UnoRuleSet};
use std::collections::HashSet;

/// Game facts only: no achievement IDs, presentation or persistence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoPlayerStatistics {
    pub uno_calls: u32,
    pub uno_reports: u32,
    pub successful_challenges: u32,
    pub challenges: u32,
    pub challenges_received: u32,
    pub successful_challenges_received: u32,
    pub uno_penalties: u32,
    pub jump_in_opportunities: u32,
    pub max_penalty_cards: u16,
    pub max_skipped_turns: u16,
    pub wild_cards: u32,
    pub jump_ins: u32,
    pub hand_swaps: u32,
    pub numbers: u16,
    pub colors: u8,
    pub has_drawn: bool,
    pub has_penalty: bool,
    pub played_penalty: bool,
    pub peak_hand: u16,
    pub reached_twenty_four: bool,
    pub draw_chain_returned: bool,
    pub max_skip_batch: u16,
    pub full_draw_chain: bool,
    pub jumped_reverse: bool,
    pub lucky_elimination: bool,
    pub max_uno_call_run: u16,
    pub finished_with_pair: bool,
    pub finished_without_uno: bool,
    pub finished_with_color_gift: bool,
}

/// One accepted transition's private report. Cumulative counters are accompanied
/// by deltas so callers can maintain lifetime progress without recounting a game.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoActionStatistics {
    pub player: UnoPlayerId,
    pub rules: UnoRuleSet,
    pub progress: UnoPlayerStatistics,
    pub wild_cards: u32,
    pub jump_ins: u32,
    pub hand_swaps: u32,
    pub completed_game: bool,
    pub won: bool,
    pub eliminated: bool,
    pub dark_side: bool,
    pub all_opponents_eliminated: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct TrackedPlayer {
    pub(super) progress: UnoPlayerStatistics,
    pub(super) called_this_turn: bool,
    pub(super) uno_call_run: u16,
    pub(super) finish: FinishCandidate,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FinishCandidate {
    pub(super) pair: bool,
    pub(super) forgotten: bool,
    pub(super) color_gift: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct DrawChain {
    pub(super) origin: Option<UnoPlayerId>,
    pub(super) links: u16,
    pub(super) cards: HashSet<UnoCard>,
}

/// The room owns one recorder for each game and invokes it only after the core
/// accepts an action. All UNO-specific interpretation stays in this crate.
#[derive(Clone, Debug)]
pub struct UnoMatchStatistics {
    pub(super) players: Vec<TrackedPlayer>,
    pub(super) chain: DrawChain,
    pub(super) color_change: Option<(UnoPlayerId, UnoColor)>,
}
