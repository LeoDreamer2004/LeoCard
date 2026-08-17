use std::fmt;

/// 所选花色比较方式判定两手牌相同后，是否允许后出的整手牌跟牌。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SameCardPolicy {
    MustBeHigher,
    CanFollow,
}

/// 同牌型且点数组成相同时，如何比较花色。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuitComparison {
    /// 只比较整手牌中最大的牌；最大牌相同即为相同强度。
    HighestCard,
    /// 从最大的牌开始逐张向下比较。
    Lexicographic,
    /// 黑桃、红桃、梅花、方块分别为 4、3、2、1 点，比较总和。
    SumPoints,
}

/// 每回合固定时间 + 每名玩家整局共享的保留时间。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeControl {
    Unlimited,
    FivePlusTen,
    FivePlusThirty,
    FifteenPlusThirty,
    ThirtyPlusSixty,
}

impl TimeControl {
    pub const OPTIONS: [Self; 5] = [
        Self::FivePlusTen,
        Self::FivePlusThirty,
        Self::FifteenPlusThirty,
        Self::ThirtyPlusSixty,
        Self::Unlimited,
    ];

    pub const fn base_seconds(self) -> u16 {
        match self {
            Self::Unlimited => 0,
            Self::FivePlusTen | Self::FivePlusThirty => 5,
            Self::FifteenPlusThirty => 15,
            Self::ThirtyPlusSixty => 30,
        }
    }

    pub const fn reserve_seconds(self) -> u16 {
        match self {
            Self::Unlimited => 0,
            Self::FivePlusTen => 10,
            Self::FivePlusThirty | Self::FifteenPlusThirty => 30,
            Self::ThirtyPlusSixty => 60,
        }
    }

    pub const fn is_unlimited(self) -> bool {
        matches!(self, Self::Unlimited)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RuleSet {
    pub deck_count: u8,
    pub player_count: u8,
    pub hand_size: u8,
    pub same_card_policy: SameCardPolicy,
    pub suit_comparison: SuitComparison,
    pub time_control: TimeControl,
    /// 启用进阶牌型及部分相同张数牌型之间的固定压制关系。
    pub advanced_play_types: bool,
    /// 仅开发构建可启用：摸牌堆只包含所有玩家的初始手牌。
    pub developer_deck: bool,
}

impl RuleSet {
    pub const MIN_DECK_COUNT: u8 = 1;
    pub const MAX_DECK_COUNT: u8 = 8;
    pub const MIN_HAND_SIZE: u8 = 5;
    pub const MAX_HAND_SIZE: u8 = 15;

    pub fn validate(self) -> Result<Self, RuleError> {
        if !(Self::MIN_DECK_COUNT..=Self::MAX_DECK_COUNT).contains(&self.deck_count) {
            return Err(RuleError::DeckCount(self.deck_count));
        }
        if !(2..=6).contains(&self.player_count) {
            return Err(RuleError::PlayerCount(self.player_count));
        }
        if !(Self::MIN_HAND_SIZE..=Self::MAX_HAND_SIZE).contains(&self.hand_size) {
            return Err(RuleError::HandSize(self.hand_size));
        }
        if self.developer_deck && !cfg!(feature = "developer") {
            return Err(RuleError::DeveloperFeatureUnavailable);
        }

        let available = usize::from(self.deck_count) * 54;
        let required = usize::from(self.player_count) * usize::from(self.hand_size);
        if required > available {
            return Err(RuleError::NotEnoughCards {
                available,
                required,
            });
        }
        Ok(self)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            deck_count: 1,
            player_count: 4,
            hand_size: 5,
            same_card_policy: SameCardPolicy::MustBeHigher,
            suit_comparison: SuitComparison::HighestCard,
            time_control: if cfg!(feature = "developer") {
                TimeControl::Unlimited
            } else {
                TimeControl::FifteenPlusThirty
            },
            advanced_play_types: false,
            developer_deck: cfg!(feature = "developer"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleError {
    DeckCount(u8),
    PlayerCount(u8),
    HandSize(u8),
    DeveloperFeatureUnavailable,
    NotEnoughCards { available: usize, required: usize },
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeckCount(value) => write!(
                f,
                "牌副数必须为 {}..={}，实际为 {value}",
                RuleSet::MIN_DECK_COUNT,
                RuleSet::MAX_DECK_COUNT
            ),
            Self::PlayerCount(value) => write!(f, "玩家数必须为 2..=6，实际为 {value}"),
            Self::HandSize(value) => write!(
                f,
                "补牌张数必须为 {}..={}，实际为 {value}",
                RuleSet::MIN_HAND_SIZE,
                RuleSet::MAX_HAND_SIZE
            ),
            Self::DeveloperFeatureUnavailable => f.write_str("当前构建没有启用开发者牌堆"),
            Self::NotEnoughCards {
                available,
                required,
            } => write!(
                f,
                "牌不足以发满初始手牌：现有 {available} 张，需要 {required} 张"
            ),
        }
    }
}

impl std::error::Error for RuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_all_configured_ranges_and_initial_capacity() {
        assert_eq!(
            RuleSet {
                deck_count: 0,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::DeckCount(0))
        );
        assert_eq!(
            RuleSet {
                deck_count: 9,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::DeckCount(9))
        );
        assert_eq!(
            RuleSet {
                player_count: 1,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::PlayerCount(1))
        );
        assert_eq!(
            RuleSet {
                player_count: 7,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::PlayerCount(7))
        );
        assert_eq!(
            RuleSet {
                hand_size: 4,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::HandSize(4))
        );
        assert_eq!(
            RuleSet {
                hand_size: 16,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::HandSize(16))
        );
        assert!(matches!(
            RuleSet {
                player_count: 6,
                hand_size: 10,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::NotEnoughCards { .. })
        ));
        assert!(
            RuleSet {
                deck_count: RuleSet::MAX_DECK_COUNT,
                hand_size: RuleSet::MAX_HAND_SIZE,
                player_count: 6,
                ..RuleSet::default()
            }
            .validate()
            .is_ok()
        );
    }
}
