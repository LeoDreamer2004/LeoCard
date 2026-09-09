//! 游戏内聊天抽屉、快捷语音与开发者手牌编辑器。

mod controls;
#[cfg(feature = "developer")]
mod developer;
mod menus;
mod panel;

use controls::*;
#[cfg(feature = "developer")]
pub(crate) use developer::*;
use menus::*;
use panel::*;

use crate::app::runtime::UiAssets;
use bevy::prelude::*;

pub(crate) struct ChatAuxiliaryAction {
    pub label: &'static str,
    pub action: Option<crate::app::shell::UiAction>,
    pub highlighted: bool,
}

pub(crate) fn add_chat_panel(
    commands: &mut Commands,
    parent: Entity,
    chat: &super::ChatPanelState,
    assets: &UiAssets,
    auto_play: Option<bool>,
    auxiliary_actions: &[ChatAuxiliaryAction],
) {
    let panel = spawn_chat_panel(commands, parent, chat);
    add_chat_toggle(commands, panel, chat, assets);
    if let Some(enabled) = auto_play {
        add_auto_play_toggle(commands, panel, enabled, assets);
    }
    add_auxiliary_actions(commands, panel, auxiliary_actions, assets);
    add_chat_history(commands, panel, chat, assets);
    add_chat_input_row(commands, panel, chat, assets);
    add_quick_voice_menu(commands, panel, chat, assets);
    add_emoji_menu(commands, panel, chat, assets);
}
