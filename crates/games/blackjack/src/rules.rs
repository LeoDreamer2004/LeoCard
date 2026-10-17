#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackRuleSet {
    pub initial_chips: u32,
    pub rounds: u8,
    pub minimum_bet: u32,
    /// 0 表示不限制真人回合时间。
    pub turn_seconds: u16,
}

impl Default for BlackjackRuleSet {
    fn default() -> Self {
        Self {
            initial_chips: 200,
            rounds: 10,
            minimum_bet: 2,
            turn_seconds: 30,
        }
    }
}

impl BlackjackRuleSet {
    pub const MIN_PLAYERS: u8 = 1;
    pub const MAX_PLAYERS: u8 = 6;
    pub const MAX_HANDS: usize = 4;

    pub const fn is_valid(self) -> bool {
        self.initial_chips >= 100
            && self.initial_chips <= 10_000
            && self.rounds > 0
            && self.rounds <= 20
            && self.minimum_bet >= 2
            && self.minimum_bet <= self.initial_chips
            && self.minimum_bet.is_multiple_of(2)
            && matches!(self.turn_seconds, 0 | 15 | 30 | 60)
    }
}
