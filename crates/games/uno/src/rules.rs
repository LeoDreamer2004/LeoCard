use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoMode {
    #[default]
    Classic,
    NoMercy,
    Flip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoNoMercyRuleSet {
    pub draw_until_playable: bool,
    pub mercy_elimination: bool,
    pub sudden_death: bool,
    pub zero_pass: bool,
    pub seven_swap: bool,
    pub uno_callout: bool,
}

impl Default for UnoNoMercyRuleSet {
    fn default() -> Self {
        Self {
            draw_until_playable: true,
            mercy_elimination: true,
            sudden_death: false,
            zero_pass: true,
            seven_swap: true,
            uno_callout: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoFlipRuleSet {
    pub random_pairing: bool,
    pub action_stacking: bool,
    pub uno_callout: bool,
    pub skip_draw_penalty: bool,
    pub jump_in: bool,
}

impl Default for UnoFlipRuleSet {
    fn default() -> Self {
        Self {
            random_pairing: false,
            action_stacking: false,
            uno_callout: true,
            skip_draw_penalty: false,
            jump_in: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoRuleSet {
    pub mode: UnoMode,
    pub action_stacking: bool,
    pub uno_callout: bool,
    pub skip_draw_penalty: bool,
    pub jump_in: bool,
    pub swap_pack: bool,
    pub reverse_pack: bool,
    pub stack_pack: bool,
    pub no_mercy: UnoNoMercyRuleSet,
    pub flip: UnoFlipRuleSet,
}

impl Default for UnoRuleSet {
    fn default() -> Self {
        Self {
            mode: UnoMode::Classic,
            action_stacking: false,
            uno_callout: true,
            skip_draw_penalty: false,
            jump_in: false,
            swap_pack: false,
            reverse_pack: false,
            stack_pack: false,
            no_mercy: UnoNoMercyRuleSet::default(),
            flip: UnoFlipRuleSet::default(),
        }
    }
}

impl UnoRuleSet {
    pub const HAND_SIZE: u8 = 7;
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 8;

    pub const fn is_classic(self) -> bool {
        matches!(self.mode, UnoMode::Classic)
    }

    pub const fn is_no_mercy(self) -> bool {
        matches!(self.mode, UnoMode::NoMercy)
    }

    pub const fn sudden_death_enabled(self) -> bool {
        self.is_no_mercy() && self.no_mercy.mercy_elimination && self.no_mercy.sudden_death
    }

    pub const fn is_flip(self) -> bool {
        matches!(self.mode, UnoMode::Flip)
    }

    /// Whether skip cards can be answered by stacking another action card.
    pub const fn action_stacking_enabled(self) -> bool {
        if self.is_flip() {
            self.flip.action_stacking
        } else {
            self.is_classic() && self.action_stacking
        }
    }

    pub const fn jump_in_enabled(self) -> bool {
        match self.mode {
            UnoMode::Classic => self.jump_in,
            UnoMode::Flip => self.flip.jump_in,
            UnoMode::NoMercy => false,
        }
    }

    pub const fn uno_callout(self) -> bool {
        match self.mode {
            UnoMode::Classic => self.uno_callout,
            UnoMode::NoMercy => self.no_mercy.uno_callout,
            UnoMode::Flip => self.flip.uno_callout,
        }
    }

    pub const fn validate(self) -> Result<Self, UnoRuleError> {
        Ok(self)
    }

    pub fn validate_player_count(player_count: u8) -> Result<u8, UnoRuleError> {
        (Self::MIN_PLAYERS..=Self::MAX_PLAYERS)
            .contains(&player_count)
            .then_some(player_count)
            .ok_or(UnoRuleError)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnoRuleError;

impl fmt::Display for UnoRuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UNO 规则无效")
    }
}

impl std::error::Error for UnoRuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_in_is_independent_from_action_stacking() {
        let rules = UnoRuleSet {
            jump_in: true,
            action_stacking: false,
            ..UnoRuleSet::default()
        };
        assert_eq!(rules.validate(), Ok(rules));
    }

    #[test]
    fn modes_keep_independent_default_rule_sets() {
        let rules = UnoRuleSet::default();
        assert_eq!(rules.mode, UnoMode::Classic);
        assert!(rules.uno_callout);
        assert!(rules.no_mercy.draw_until_playable);
        assert!(rules.no_mercy.mercy_elimination);
        assert!(!rules.no_mercy.sudden_death);
        assert!(rules.flip.uno_callout);
        assert!(!rules.flip.random_pairing);
    }
}
