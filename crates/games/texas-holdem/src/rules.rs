use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasRuleSet {
    pub player_count: u8,
    pub starting_chips: u16,
    /// Zero disables squid; otherwise at most player_count - 1 markers per round.
    pub squid_markers: u8,
    pub short_deck: bool,
    /// 只比较构成牌型的点数，忽略对子、两对、三条和四条之外的踢脚牌。
    /// 高牌与同花只比较最大的一张牌。
    #[cfg_attr(feature = "serde", serde(default))]
    pub ignore_kickers: bool,
    /// 奥马哈高牌：每人四张底牌，成牌时必须恰好使用两张底牌和三张公共牌。
    #[cfg_attr(feature = "serde", serde(default))]
    pub omaha: bool,
    /// 固定限注：前两街每次 2，后两街每次 4，每街最多四次完整下注。
    #[cfg_attr(feature = "serde", serde(default))]
    pub fixed_limit: bool,
    /// DeepSeek squid markers cost 2BB instead of 1BB when squid is enabled.
    pub deepseek_squid_double: bool,
}

impl TexasRuleSet {
    pub const MIN_PLAYERS: u8 = 3;
    pub const MAX_PLAYERS: u8 = 8;
    pub const STARTING_CHIP_OPTIONS: [u16; 11] = [5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    pub const SMALL_BLIND: u32 = 1;
    pub const BIG_BLIND: u32 = 2;
    pub const MAX_FIXED_LIMIT_BETS: u8 = 4;

    pub const fn hole_card_count(self) -> usize {
        if self.omaha { 4 } else { 2 }
    }

    pub const fn supports_player_count(self, player_count: u8) -> bool {
        player_count >= Self::MIN_PLAYERS
            && player_count <= Self::MAX_PLAYERS
            && !(player_count > 6 && self.short_deck && self.omaha)
    }

    pub fn validate(self) -> Result<Self, TexasRuleError> {
        self.validate_options()?;
        if !self.supports_player_count(self.player_count) {
            return Err(TexasRuleError::ShortDeckOmahaPlayerCount(self.player_count));
        }
        Ok(self)
    }

    /// Validate lobby options independently of the room's seating capacity.
    /// The actual participant count is validated again when starting a hand.
    pub fn validate_options(self) -> Result<Self, TexasRuleError> {
        if !(Self::MIN_PLAYERS..=Self::MAX_PLAYERS).contains(&self.player_count) {
            return Err(TexasRuleError::PlayerCount(self.player_count));
        }
        if !Self::STARTING_CHIP_OPTIONS.contains(&self.starting_chips) {
            return Err(TexasRuleError::StartingChips(self.starting_chips));
        }
        if self.squid_markers >= self.player_count {
            return Err(TexasRuleError::SquidMarkers(self.squid_markers));
        }
        Ok(self)
    }
}

impl Default for TexasRuleSet {
    fn default() -> Self {
        Self {
            player_count: 4,
            starting_chips: 20,
            squid_markers: 0,
            short_deck: false,
            ignore_kickers: false,
            omaha: false,
            fixed_limit: false,
            deepseek_squid_double: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TexasRuleError {
    PlayerCount(u8),
    ShortDeckOmahaPlayerCount(u8),
    StartingChips(u16),
    SquidMarkers(u8),
}

impl fmt::Display for TexasRuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SquidMarkers(value) => write!(f, "鱿鱼标记数必须少于玩家人数，实际为 {value}"),
            Self::PlayerCount(value) => write!(f, "玩家数必须为 3..=8，实际为 {value}"),
            Self::ShortDeckOmahaPlayerCount(value) => {
                write!(f, "短牌奥马哈最多支持 6 人，实际为 {value}")
            }
            Self::StartingChips(value) => {
                write!(f, "初始筹码必须是 5 或 10 至 100 的整十数，实际为 {value}")
            }
        }
    }
}

impl std::error::Error for TexasRuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_player_and_starting_chip_options() {
        assert!(!TexasRuleSet::default().ignore_kickers);
        assert!(!TexasRuleSet::default().omaha);
        assert_eq!(TexasRuleSet::default().hole_card_count(), 2);
        assert_eq!(
            TexasRuleSet {
                omaha: true,
                ..TexasRuleSet::default()
            }
            .hole_card_count(),
            4
        );
        assert_eq!(TexasRuleSet::default().starting_chips, 20);
        assert_eq!(TexasRuleSet::STARTING_CHIP_OPTIONS.last(), Some(&100));
        for starting_chips in TexasRuleSet::STARTING_CHIP_OPTIONS {
            assert!(
                TexasRuleSet {
                    starting_chips,
                    ..TexasRuleSet::default()
                }
                .validate()
                .is_ok()
            );
        }
        assert_eq!(
            TexasRuleSet {
                player_count: 2,
                ..TexasRuleSet::default()
            }
            .validate(),
            Err(TexasRuleError::PlayerCount(2))
        );
        assert_eq!(
            TexasRuleSet {
                starting_chips: 15,
                ..TexasRuleSet::default()
            }
            .validate(),
            Err(TexasRuleError::StartingChips(15))
        );
        assert_eq!(
            TexasRuleSet {
                starting_chips: 110,
                ..TexasRuleSet::default()
            }
            .validate(),
            Err(TexasRuleError::StartingChips(110))
        );
    }
}
