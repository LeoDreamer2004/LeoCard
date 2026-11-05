use super::super::{MusicState, MusicUiAction};
use super::widgets::{button, row};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn render_account(
    commands: &mut UiComposer,
    parent: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    add_title(commands, parent, "网易云音乐", 22.0, TEXT, assets);
    add_text(commands, parent, &state.status, 15.0, TEXT, assets);
    if let Some(qr) = &state.qr_image {
        commands.child(
            parent,
            (
                Node {
                    width: px(180),
                    height: px(180),
                    flex_shrink: 0.0,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ImageNode::new(qr.clone()),
            ),
        );
    }
    let controls = row(commands, parent);
    if state.account.is_some() {
        button(
            commands,
            controls,
            "刷新歌单",
            MusicUiAction::Refresh,
            state.catalog_task.is_none(),
            assets,
        );
    } else {
        button(
            commands,
            controls,
            if state.qr_image.is_some() {
                "刷新二维码"
            } else {
                "扫码关联"
            },
            MusicUiAction::Login,
            state.account_task.is_none(),
            assets,
        );
    }
    button(
        commands,
        controls,
        if state.remember {
            "记住账号 已开启"
        } else {
            "记住账号 已关闭"
        },
        MusicUiAction::Remember,
        true,
        assets,
    );
    if state.account.is_some() || state.saved.is_some() || state.qr_image.is_some() {
        button(
            commands,
            controls,
            "解除关联",
            MusicUiAction::Forget,
            true,
            assets,
        );
    }
    add_text(
        commands,
        parent,
        if state.remember {
            "登录凭据仅保存在本机，解除关联会清除。"
        } else {
            "仅本次登录，关闭游戏后需重新扫码。"
        },
        13.0,
        MUTED,
        assets,
    );
    if state.qr_image.is_some() && cfg!(target_os = "android") {
        add_text(
            commands,
            parent,
            "同一手机可截图后，在网易云音乐中从相册识别二维码。",
            13.0,
            MUTED,
            assets,
        );
    }
}
