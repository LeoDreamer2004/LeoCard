use super::super::UnoUiAction;
use crate::app::presentation::UiComposer;
use crate::app::runtime::UiAssets;
use crate::app::shell::{CozyDropdown, CozyDropdownOption, UiAction};
use bevy::prelude::*;
use leocard_uno::{Mode, UnoRuleSet};

pub(crate) fn render_uno_mode_dropdown(
    commands: &mut UiComposer,
    root: Entity,
    parent: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    open: bool,
    assets: &UiAssets,
) {
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
    .render(commands, root, parent, can_configure, open, assets);
}
