#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DoudizhuMode {
    #[default]
    Three,
    Four,
}

impl DoudizhuMode {
    pub const fn players(self) -> u8 {
        match self {
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    pub const fn deck_count(self) -> u8 {
        match self {
            Self::Three => 1,
            Self::Four => 2,
        }
    }

    pub const fn kitty_len(self) -> usize {
        match self {
            Self::Three => 3,
            Self::Four => 8,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Three => "三人斗地主",
            Self::Four => "四人斗地主",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoudizhuRuleSet {
    pub mode: DoudizhuMode,
    pub rounds: u8,
    /// 四人玩法允许三带一及飞机带单张；三人玩法始终允许。
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow_single_wings: bool,
}

impl Default for DoudizhuRuleSet {
    fn default() -> Self {
        Self {
            mode: DoudizhuMode::Three,
            rounds: 1,
            allow_single_wings: false,
        }
    }
}

impl DoudizhuRuleSet {
    pub fn for_mode(mode: DoudizhuMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    pub const fn allows_single_wings(self) -> bool {
        matches!(self.mode, DoudizhuMode::Three) || self.allow_single_wings
    }

    pub const fn is_valid(self) -> bool {
        self.rounds >= 1 && self.rounds <= 5
    }
}
