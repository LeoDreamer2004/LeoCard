use super::{TexasGameError, TexasGameState, TexasPhase};
use crate::TexasRuleSet;
use leocard_game_common::PlayerId;

/// Ordinary raises advance from the current bet in multiples of the last full raise.
/// An all-in is a separate action and may fall between these steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TexasRaiseSizing {
    current: u32,
    increment: u32,
    limit: u32,
}

impl TexasGameState {
    /// 统一提供给规则校验、机器人与客户端的普通加注范围。
    pub fn raise_sizing(&self, player: PlayerId) -> Result<TexasRaiseSizing, TexasGameError> {
        let state = self
            .players
            .get(player.index())
            .ok_or(TexasGameError::InvalidPlayer(player))?;
        let mut maximum = state.committed_street.saturating_add(state.stack);
        if !self.raise_allowed(player)? {
            maximum = self.current_bet;
        } else if self.rules.fixed_limit {
            maximum = maximum.min(self.minimum_raise_to());
        }
        Ok(TexasRaiseSizing::new(
            self.current_bet,
            self.minimum_raise,
            maximum,
        ))
    }

    pub fn all_in_allowed(&self, player: PlayerId) -> Result<bool, TexasGameError> {
        let state = self
            .players
            .get(player.index())
            .ok_or(TexasGameError::InvalidPlayer(player))?;
        if !self.can_act(player)
            || self.pending_blind.is_some()
            || !matches!(self.phase, TexasPhase::Betting(_))
        {
            return Ok(false);
        }
        let target = state.committed_street.saturating_add(state.stack);
        Ok(target <= self.current_bet
            || (self.raise_allowed(player)?
                && (!self.rules.fixed_limit || target <= self.minimum_raise_to())))
    }

    pub(super) fn fixed_limit_reached(&self) -> bool {
        self.rules.fixed_limit && self.full_bets >= TexasRuleSet::MAX_FIXED_LIMIT_BETS
    }

    pub(super) fn validate_raise_rights(&self, player: PlayerId) -> Result<(), TexasGameError> {
        if self.fixed_limit_reached() {
            return Err(TexasGameError::RaiseLimitReached);
        }
        if !self.raise_allowed[player.index()] {
            return Err(TexasGameError::RaiseNotReopened);
        }
        Ok(())
    }

    pub(super) fn validate_fixed_limit_target(&self, target: u32) -> Result<(), TexasGameError> {
        let maximum_target = self.minimum_raise_to();
        if self.rules.fixed_limit && target > maximum_target {
            return Err(TexasGameError::RaiseExceedsLimit {
                maximum_target,
                target,
            });
        }
        Ok(())
    }

    pub(super) fn record_full_raise(&mut self, target: u32) {
        if self.rules.fixed_limit {
            self.full_bets += 1;
        } else {
            self.minimum_raise = target - self.current_bet;
        }
    }
}

impl TexasRaiseSizing {
    pub fn new(current: u32, increment: u32, limit: u32) -> Self {
        Self {
            current,
            increment: increment.max(1),
            limit,
        }
    }

    pub fn minimum(self) -> u32 {
        self.current.saturating_add(self.increment)
    }

    pub fn maximum(self) -> Option<u32> {
        let steps = self.limit.saturating_sub(self.current) / self.increment;
        (steps > 0).then(|| self.current + steps * self.increment)
    }

    pub fn contains(self, target: u32) -> bool {
        target >= self.minimum()
            && target <= self.limit
            && (target - self.current).is_multiple_of(self.increment)
    }

    /// Round up to a step, stopping at the largest affordable ordinary raise.
    pub fn normalize(self, target: u32) -> Option<u32> {
        let maximum = self.maximum()?;
        let steps = target
            .clamp(self.minimum(), maximum)
            .saturating_sub(self.current)
            .div_ceil(self.increment);
        Some((self.current + steps * self.increment).min(maximum))
    }
}
