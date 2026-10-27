#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    #[default]
    Three,
    Four,
}

impl Mode {
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
pub struct RuleSet {
    pub mode: Mode,
    pub rounds: u8,
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            mode: Mode::Three,
            rounds: 1,
        }
    }
}

impl RuleSet {
    pub const fn is_valid(self) -> bool {
        self.rounds >= 1 && self.rounds <= 5
    }
}
