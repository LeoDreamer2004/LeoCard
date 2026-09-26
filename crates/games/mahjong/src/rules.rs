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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongUmaStyle {
    #[default]
    Balanced,
    FirstPlace,
    AvoidFourth,
}

impl MahjongUmaStyle {
    pub const fn points(self, length: MahjongMatchLength) -> [i16; 4] {
        match (length, self) {
            (MahjongMatchLength::SingleHand, _) => [0; 4],
            (MahjongMatchLength::EastRound, Self::Balanced) => [3, 1, -1, -3],
            (MahjongMatchLength::EastRound, Self::FirstPlace) => [5, 0, -2, -3],
            (MahjongMatchLength::EastRound, Self::AvoidFourth) => [3, 1, 0, -4],
            (MahjongMatchLength::HalfGame, Self::Balanced) => [6, 3, -3, -6],
            (MahjongMatchLength::HalfGame, Self::FirstPlace) => [11, 1, -4, -6],
            (MahjongMatchLength::HalfGame, Self::AvoidFourth) => [6, 3, 0, -9],
            (MahjongMatchLength::FullGame, Self::Balanced) => [15, 4, -4, -15],
            (MahjongMatchLength::FullGame, Self::FirstPlace) => [20, 3, -8, -15],
            (MahjongMatchLength::FullGame, Self::AvoidFourth) => [15, 7, -2, -20],
        }
    }
}

/// 四人最终分数换算为档案积分。并列时按座次决定马点名次。
pub fn mahjong_reference_deltas(scores: &[i32; 4], rules: MahjongRuleSet) -> [i16; 4] {
    let divisor = match rules.match_length {
        MahjongMatchLength::SingleHand => 8,
        MahjongMatchLength::EastRound => 16,
        MahjongMatchLength::HalfGame => 24,
        MahjongMatchLength::FullGame => 32,
    };
    let uma = rules.uma_style.points(rules.match_length);
    let mut order = [0, 1, 2, 3];
    order.sort_by_key(|index| (std::cmp::Reverse(scores[*index]), *index));
    let mut deltas = [0; 4];
    for (rank, player) in order.into_iter().enumerate() {
        deltas[player] = (scores[player] / divisor) as i16 + uma[rank];
    }
    deltas
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongRuleSet {
    pub match_length: MahjongMatchLength,
    pub uma_style: MahjongUmaStyle,
    pub minimum_eight_points: bool,
    pub multiple_winners: bool,
    pub false_win: bool,
}

impl Default for MahjongRuleSet {
    fn default() -> Self {
        Self {
            match_length: MahjongMatchLength::SingleHand,
            uma_style: MahjongUmaStyle::Balanced,
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

    #[test]
    fn reference_deltas_follow_length_and_uma() {
        let mut rules = MahjongRuleSet::default();
        assert_eq!(
            mahjong_reference_deltas(&[-20, 36, -8, -8], rules),
            [-2, 4, -1, -1]
        );
        rules.match_length = MahjongMatchLength::EastRound;
        assert_eq!(
            mahjong_reference_deltas(&[100, -24, -96, 20], rules),
            [9, -2, -9, 2]
        );
        rules.uma_style = MahjongUmaStyle::FirstPlace;
        assert_eq!(
            mahjong_reference_deltas(&[100, -24, -96, 20], rules),
            [11, -3, -9, 1]
        );
        rules.uma_style = MahjongUmaStyle::AvoidFourth;
        assert_eq!(
            mahjong_reference_deltas(&[100, -24, -96, 20], rules),
            [9, -1, -10, 2]
        );
        for (length, style, expected) in [
            (
                MahjongMatchLength::HalfGame,
                MahjongUmaStyle::Balanced,
                [10, -4, -10, 3],
            ),
            (
                MahjongMatchLength::HalfGame,
                MahjongUmaStyle::FirstPlace,
                [15, -5, -10, 1],
            ),
            (
                MahjongMatchLength::HalfGame,
                MahjongUmaStyle::AvoidFourth,
                [10, -1, -13, 3],
            ),
            (
                MahjongMatchLength::FullGame,
                MahjongUmaStyle::Balanced,
                [18, -4, -18, 4],
            ),
            (
                MahjongMatchLength::FullGame,
                MahjongUmaStyle::FirstPlace,
                [23, -8, -18, 3],
            ),
            (
                MahjongMatchLength::FullGame,
                MahjongUmaStyle::AvoidFourth,
                [18, -2, -23, 7],
            ),
        ] {
            rules.match_length = length;
            rules.uma_style = style;
            assert_eq!(
                mahjong_reference_deltas(&[100, -24, -96, 20], rules),
                expected
            );
        }
    }
}
