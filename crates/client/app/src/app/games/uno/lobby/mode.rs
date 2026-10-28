use super::super::UnoUiAction;
use crate::app::shell::{CozyDropdown, CozyDropdownOption, UiAction};
use leocard_uno::{Mode, UnoRuleSet};

pub(super) fn uno_mode_dropdown(rules: UnoRuleSet) -> CozyDropdown {
    let modes = [
        (Mode::Classic, "UNO"),
        (Mode::NoMercy, "No Mercy"),
        (Mode::Flip, "UNO FLIP"),
    ];
    CozyDropdown {
        label: modes
            .iter()
            .find(|(mode, _)| *mode == rules.mode)
            .expect("UNO mode")
            .1,
        toggle_action: UiAction::Uno(UnoUiAction::ToggleModeMenu),
        close_action: UiAction::Uno(UnoUiAction::CloseModeMenu),
        options: modes
            .into_iter()
            .map(|(mode, label)| CozyDropdownOption {
                label,
                selected: mode == rules.mode,
                action: UiAction::Uno(UnoUiAction::UpdateRules(UnoRuleSet { mode, ..rules })),
            })
            .collect(),
    }
}
