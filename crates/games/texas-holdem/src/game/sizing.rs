use super::{GameError, GameState, Phase, TexasHoldemPlayerId};
use crate::TexasHoldemRuleSet;

/// Ordinary raises advance from the current bet in multiples of the last full raise.
/// An all-in is a separate action and may fall between these steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaiseSizing {
    current: u32,
    increment: u32,
    limit: u32,
}

impl GameState {
    /// 统一提供给规则校验、机器人与客户端的普通加注范围。
    pub fn raise_sizing(&self, player: TexasHoldemPlayerId) -> Result<RaiseSizing, GameError> {
        let state = self
            .players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        let mut maximum = state.committed_street.saturating_add(state.stack);
        if !self.raise_allowed(player)? {
            maximum = self.current_bet;
        } else if self.rules.fixed_limit {
            maximum = maximum.min(self.minimum_raise_to());
        }
        Ok(RaiseSizing::new(
            self.current_bet,
            self.minimum_raise,
            maximum,
        ))
    }

    pub fn all_in_allowed(&self, player: TexasHoldemPlayerId) -> Result<bool, GameError> {
        let state = self
            .players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        if !self.can_act(player)
            || self.pending_blind.is_some()
            || !matches!(self.phase, Phase::Betting(_))
        {
            return Ok(false);
        }
        let target = state.committed_street.saturating_add(state.stack);
        Ok(target <= self.current_bet
            || (self.raise_allowed(player)?
                && (!self.rules.fixed_limit || target <= self.minimum_raise_to())))
    }

    pub(super) fn fixed_limit_reached(&self) -> bool {
        self.rules.fixed_limit && self.full_bets >= TexasHoldemRuleSet::MAX_FIXED_LIMIT_BETS
    }

    pub(super) fn validate_raise_rights(
        &self,
        player: TexasHoldemPlayerId,
    ) -> Result<(), GameError> {
        if self.fixed_limit_reached() {
            return Err(GameError::RaiseLimitReached);
        }
        if !self.raise_allowed[player.0] {
            return Err(GameError::RaiseNotReopened);
        }
        Ok(())
    }

    pub(super) fn validate_fixed_limit_target(&self, target: u32) -> Result<(), GameError> {
        let maximum_target = self.minimum_raise_to();
        if self.rules.fixed_limit && target > maximum_target {
            return Err(GameError::RaiseExceedsLimit {
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

impl RaiseSizing {
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
