use super::super::ShengjiUiAction;
use crate::app::shell::{CozyDropdown, CozyDropdownOption, UiAction};
use leocard_shengji::{ShengjiMode, ShengjiRuleSet};

pub(super) fn mode_dropdown(rules_value: ShengjiRuleSet) -> CozyDropdown {
    CozyDropdown {
        label: rules_value.mode.label(),
        toggle_action: UiAction::Shengji(ShengjiUiAction::ToggleModeMenu),
        close_action: UiAction::Shengji(ShengjiUiAction::CloseModeMenu),
        options: [ShengjiMode::Classic, ShengjiMode::SixPlayer]
            .into_iter()
            .map(|mode| CozyDropdownOption {
                label: mode.label(),
                selected: mode == rules_value.mode,
                action: UiAction::Shengji(ShengjiUiAction::SelectMode(mode)),
            })
            .collect(),
    }
}
