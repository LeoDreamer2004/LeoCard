//! 终局结算后的通用房间操作。

use crate::app::presentation::UiComposer;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, UiAction, add_cozy_button, add_cozy_button_variant,
    add_cozy_disabled_button,
};
use bevy::prelude::*;

pub(crate) struct SummaryActions<'a> {
    pub is_host: bool,
    pub ready: bool,
    pub again: &'a str,
}

pub(crate) fn add_summary_actions(
    commands: &mut UiComposer,
    parent: Entity,
    state: SummaryActions<'_>,
    assets: &UiAssets,
) {
    add_cozy_button_variant(
        commands,
        parent,
        "退出游戏",
        UiAction::Lobby(LobbyUiAction::LeaveRoom),
        assets,
        px(150),
        48.0,
        CozyButtonVariant::Danger,
    );
    if state.is_host {
        add_cozy_button(
            commands,
            parent,
            "返回房间",
            UiAction::Lobby(LobbyUiAction::ReturnToLobby),
            assets,
            px(150),
            48.0,
        );
    }
    if state.ready {
        add_cozy_disabled_button(commands, parent, "已准备", assets, px(150), 48.0);
    } else {
        add_cozy_button_variant(
            commands,
            parent,
            state.again,
            UiAction::Lobby(LobbyUiAction::PlayAgain),
            assets,
            px(150),
            48.0,
            CozyButtonVariant::Cool,
        );
    }
}
