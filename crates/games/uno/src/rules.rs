use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RuleSet {
    /// 是否允许把万能摸四叠在摸二上；默认关闭。
    pub stack_draw_four_on_draw_two: bool,
    /// 是否启用 UNO 宣告、检举和漏喊罚两张。
    pub uno_callout: bool,
    /// 被禁手的玩家每实际跳过一轮时是否额外摸一张。
    pub skip_draw_penalty: bool,
    /// 被禁手的玩家是否可以用另一张禁手牌把累计禁手转给下一位。
    pub stack_skip: bool,
    /// 是否允许非下家用完全相同的非万能牌抢出；依赖禁手叠加。
    pub jump_in: bool,
}

impl RuleSet {
    pub const HAND_SIZE: u8 = 7;
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 6;

    pub fn validate(self) -> Result<Self, RuleError> {
        if self.jump_in && !self.stack_skip {
            Err(RuleError)
        } else {
            Ok(self)
        }
    }

    pub fn validate_player_count(player_count: u8) -> Result<usize, RuleError> {
        (Self::MIN_PLAYERS..=Self::MAX_PLAYERS)
            .contains(&player_count)
            .then_some(usize::from(player_count))
            .ok_or(RuleError)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            stack_draw_four_on_draw_two: false,
            uno_callout: true,
            skip_draw_penalty: false,
            stack_skip: false,
            jump_in: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleError;

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UNO 规则无效")
    }
}

impl std::error::Error for RuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_in_is_disabled_by_default_and_requires_skip_stacking() {
        assert!(!RuleSet::default().jump_in);
        assert!(
            RuleSet {
                jump_in: true,
                ..RuleSet::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            RuleSet {
                stack_skip: true,
                jump_in: true,
                ..RuleSet::default()
            }
            .validate()
            .is_ok()
        );
    }
}
