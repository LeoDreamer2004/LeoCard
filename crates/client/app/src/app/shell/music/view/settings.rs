use super::super::{MusicState, MusicUiAction};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{CozyCheckbox, NavigationUiAction, UiAction, add_cozy_button};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::UiComposer;

pub(crate) fn render_music_settings(
    commands: &mut UiComposer,
    parent: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    add_title(commands, parent, "音乐播放器", 22.0, TEXT, assets);
    let button = commands.child(
        parent,
        (
            Button,
            UiAction::Music(MusicUiAction::ToggleInGame),
            Node {
                width: percent(100),
                min_height: px(48),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        ),
    );
    let icon = commands.child(
        button,
        (
            Node {
                width: px(34),
                height: px(34),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(if state.show_in_game {
                assets.home.checkbox_selected.clone()
            } else {
                assets.home.checkbox.clone()
            }),
            Pickable::IGNORE,
        ),
    );
    commands.entity(button).insert(CozyCheckbox {
        icon,
        selected: state.show_in_game,
    });
    add_text(
        commands,
        button,
        "在游戏中显示小型播放器",
        16.0,
        TEXT,
        assets,
    );
    add_text(
        commands,
        parent,
        "默认显示在对局左上角，拖动顶部歌名可以移动。隐藏后音乐继续播放。",
        14.0,
        MUTED,
        assets,
    );
    add_cozy_button(
        commands,
        parent,
        "打开播放器",
        UiAction::Navigation(NavigationUiAction::ToggleMusic),
        assets,
        px(160),
        44.0,
    );
    add_text(
        commands,
        parent,
        "也可以通过右上角音乐图标关联账号、选择歌单和歌曲。",
        14.0,
        MUTED,
        assets,
    );
}
