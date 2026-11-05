#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuandanRuleSet {
    /// The rank both teams play in the opening hand (2 through ace).
    pub starting_level: u8,
}

impl GuandanRuleSet {
    pub const PLAYER_COUNT: u8 = 4;

    pub const fn is_valid(self) -> bool {
        self.starting_level >= 2 && self.starting_level <= 14
    }
}

impl Default for GuandanRuleSet {
    fn default() -> Self {
        Self { starting_level: 2 }
    }
}
