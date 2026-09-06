use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasHoldemRuleSet {
    pub player_count: u8,
    pub starting_chips: u16,
    pub short_deck: bool,
    /// 只比较构成牌型的点数，忽略对子、两对、三条和四条之外的踢脚牌。
    /// 高牌与同花只比较最大的一张牌。
    #[cfg_attr(feature = "serde", serde(default))]
    pub ignore_kickers: bool,
    /// 奥马哈高牌：每人四张底牌，成牌时必须恰好使用两张底牌和三张公共牌。
    #[cfg_attr(feature = "serde", serde(default))]
    pub omaha: bool,
}

impl TexasHoldemRuleSet {
    pub const MIN_PLAYERS: u8 = 3;
    pub const MAX_PLAYERS: u8 = 6;
    pub const STARTING_CHIP_OPTIONS: [u16; 6] = [5, 10, 20, 30, 40, 50];
    pub const SMALL_BLIND: u32 = 1;
    pub const BIG_BLIND: u32 = 2;

    pub const fn hole_card_count(self) -> usize {
        if self.omaha { 4 } else { 2 }
    }

    pub fn validate(self) -> Result<Self, RuleError> {
        if !(Self::MIN_PLAYERS..=Self::MAX_PLAYERS).contains(&self.player_count) {
            return Err(RuleError::PlayerCount(self.player_count));
        }
        if !Self::STARTING_CHIP_OPTIONS.contains(&self.starting_chips) {
            return Err(RuleError::StartingChips(self.starting_chips));
        }
        Ok(self)
    }
}

impl Default for TexasHoldemRuleSet {
    fn default() -> Self {
        Self {
            player_count: 4,
            starting_chips: 20,
            short_deck: false,
            ignore_kickers: false,
            omaha: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleError {
    PlayerCount(u8),
    StartingChips(u16),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerCount(value) => write!(f, "玩家数必须为 3..=6，实际为 {value}"),
            Self::StartingChips(value) => {
                write!(f, "初始筹码必须是 5、10、20、30、40 或 50，实际为 {value}")
            }
        }
    }
}

impl std::error::Error for RuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_player_and_starting_chip_options() {
        assert!(!TexasHoldemRuleSet::default().ignore_kickers);
        assert!(!TexasHoldemRuleSet::default().omaha);
        assert_eq!(TexasHoldemRuleSet::default().hole_card_count(), 2);
        assert_eq!(
            TexasHoldemRuleSet {
                omaha: true,
                ..TexasHoldemRuleSet::default()
            }
            .hole_card_count(),
            4
        );
        assert_eq!(TexasHoldemRuleSet::default().starting_chips, 20);
        for starting_chips in TexasHoldemRuleSet::STARTING_CHIP_OPTIONS {
            assert!(
                TexasHoldemRuleSet {
                    starting_chips,
                    ..TexasHoldemRuleSet::default()
                }
                .validate()
                .is_ok()
            );
        }
        assert_eq!(
            TexasHoldemRuleSet {
                player_count: 2,
                ..TexasHoldemRuleSet::default()
            }
            .validate(),
            Err(RuleError::PlayerCount(2))
        );
        assert_eq!(
            TexasHoldemRuleSet {
                starting_chips: 15,
                ..TexasHoldemRuleSet::default()
            }
            .validate(),
            Err(RuleError::StartingChips(15))
        );
    }
}
