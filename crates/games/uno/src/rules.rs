use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    #[default]
    Classic,
    NoMercy,
    Flip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoMercyRuleSet {
    pub draw_until_playable: bool,
    pub mercy_elimination: bool,
    pub zero_pass: bool,
    pub seven_swap: bool,
    pub uno_callout: bool,
}

impl Default for NoMercyRuleSet {
    fn default() -> Self {
        Self {
            draw_until_playable: true,
            mercy_elimination: true,
            zero_pass: true,
            seven_swap: true,
            uno_callout: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlipRuleSet {
    pub random_pairing: bool,
    pub action_stacking: bool,
    pub uno_callout: bool,
    pub skip_draw_penalty: bool,
    pub jump_in: bool,
}

impl Default for FlipRuleSet {
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
    pub mode: Mode,
    pub action_stacking: bool,
    pub uno_callout: bool,
    pub skip_draw_penalty: bool,
    pub jump_in: bool,
    pub swap_pack: bool,
    pub reverse_pack: bool,
    pub stack_pack: bool,
    pub no_mercy: NoMercyRuleSet,
    pub flip: FlipRuleSet,
}

impl Default for UnoRuleSet {
    fn default() -> Self {
        Self {
            mode: Mode::Classic,
            action_stacking: false,
            uno_callout: true,
            skip_draw_penalty: false,
            jump_in: false,
            swap_pack: false,
            reverse_pack: false,
            stack_pack: false,
            no_mercy: NoMercyRuleSet::default(),
            flip: FlipRuleSet::default(),
        }
    }
}

impl UnoRuleSet {
    pub const HAND_SIZE: u8 = 7;
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 6;

    pub const fn is_classic(self) -> bool {
        matches!(self.mode, Mode::Classic)
    }

    pub const fn is_no_mercy(self) -> bool {
        matches!(self.mode, Mode::NoMercy)
    }

    pub const fn is_flip(self) -> bool {
        matches!(self.mode, Mode::Flip)
    }

    pub const fn uno_callout(self) -> bool {
        match self.mode {
            Mode::Classic => self.uno_callout,
            Mode::NoMercy => self.no_mercy.uno_callout,
            Mode::Flip => self.flip.uno_callout,
        }
    }

    pub const fn validate(self) -> Result<Self, RuleError> {
        Ok(self)
    }

    pub fn validate_player_count(player_count: u8) -> Result<usize, RuleError> {
        (Self::MIN_PLAYERS..=Self::MAX_PLAYERS)
            .contains(&player_count)
            .then_some(usize::from(player_count))
            .ok_or(RuleError)
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
        assert_eq!(rules.mode, Mode::Classic);
        assert!(rules.uno_callout);
        assert!(rules.no_mercy.draw_until_playable);
        assert!(rules.no_mercy.mercy_elimination);
        assert!(rules.flip.uno_callout);
        assert!(!rules.flip.random_pairing);
    }
}
