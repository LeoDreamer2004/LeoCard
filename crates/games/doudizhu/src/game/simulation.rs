use super::{DoudizhuGameState, DoudizhuPhase};
use crate::{DoudizhuCard, DoudizhuPlay, DoudizhuRuleSet};
use leocard_game_common::PlayerId;

impl DoudizhuGameState {
    /// Internal rollout position built exclusively from a sampled deal, never a live clone.
    /// Scoring history is intentionally absent: rollouts evaluate team wins, not multipliers.
    pub(crate) fn sampled_position(
        rules: DoudizhuRuleSet,
        hands: Vec<Vec<DoudizhuCard>>,
        current: PlayerId,
        landlord: PlayerId,
        table: Option<(PlayerId, DoudizhuPlay)>,
    ) -> Self {
        let count = usize::from(rules.mode.players());
        let passes = table.as_ref().map_or(0, |(leader, _)| {
            (current.0 + rules.mode.players() - leader.0 - 1) % rules.mode.players()
        });
        Self {
            rules,
            phase: DoudizhuPhase::Playing,
            current,
            landlord: Some(landlord),
            highest_bid: 1,
            bids: vec![None; count],
            table,
            played: Vec::new(),
            last_actions: vec![None; count],
            multiplier: 1,
            result: None,
            hands,
            kitty: Vec::new(),
            passes,
            play_counts: vec![0; count],
        }
    }
}
