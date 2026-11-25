#[cfg(feature = "desktop")]
use super::section::add_settings_separator;
use super::toggle::add_settings_toggle;
use crate::app::presentation::{TEXT, add_title};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use crate::app::shell::SettingsUiAction;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn render_notification_settings(
    commands: &mut UiComposer,
    parent: Entity,
    form: &AppearancePreferences,
    assets: &UiAssets,
) {
    #[cfg(feature = "desktop")]
    {
        add_title(commands, parent, "弹窗", 20.0, TEXT, assets);
        add_settings_toggle(
            commands,
            parent,
            "轮到我时弹窗提醒",
            form.turn_notifications,
            SettingsUiAction::ToggleTurnNotifications,
            assets,
        );
        add_settings_separator(commands, parent);
    }
    add_title(commands, parent, "成就", 20.0, TEXT, assets);
    for (label, selected, action) in [
        (
            "弹出自己的成就",
            form.own_achievement_notifications,
            SettingsUiAction::ToggleOwnAchievementNotifications,
        ),
        (
            "弹出他人成就",
            form.other_achievement_notifications,
            SettingsUiAction::ToggleOtherAchievementNotifications,
        ),
    ] {
        add_settings_toggle(commands, parent, label, selected, action, assets);
    }
}
