use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongPlayerId(pub usize);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongMatchLength {
    #[default]
    SingleHand,
    EastRound,
    HalfGame,
    FullGame,
}

impl MahjongMatchLength {
    pub const fn hand_count(self) -> u8 {
        match self {
            Self::SingleHand => 1,
            Self::EastRound => 4,
            Self::HalfGame => 8,
            Self::FullGame => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongRuleSet {
    pub match_length: MahjongMatchLength,
    pub minimum_eight_points: bool,
    pub multiple_winners: bool,
    pub false_win: bool,
}

impl Default for MahjongRuleSet {
    fn default() -> Self {
        Self {
            match_length: MahjongMatchLength::SingleHand,
            minimum_eight_points: true,
            multiple_winners: false,
            false_win: false,
        }
    }
}

impl MahjongRuleSet {
    pub const PLAYER_COUNT: usize = 4;

    pub const fn validate(self) -> Result<Self, RuleError> {
        if self.false_win && !self.minimum_eight_points {
            return Err(RuleError::FalseWinRequiresMinimum);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleError {
    FalseWinRequiresMinimum,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FalseWinRequiresMinimum => f.write_str("允许错和依赖 8 番起和"),
        }
    }
}

impl std::error::Error for RuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_one_hand_with_standard_minimum() {
        let rules = MahjongRuleSet::default();
        assert_eq!(rules.match_length.hand_count(), 1);
        assert!(rules.minimum_eight_points);
        assert!(!rules.multiple_winners);
        assert!(!rules.false_win);
    }

    #[test]
    fn false_win_requires_the_minimum() {
        let rules = MahjongRuleSet {
            minimum_eight_points: false,
            false_win: true,
            ..MahjongRuleSet::default()
        };
        assert_eq!(rules.validate(), Err(RuleError::FalseWinRequiresMinimum));
    }
}
