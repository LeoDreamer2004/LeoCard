use super::super::mode::PlayMode;
use super::super::{MusicState, MusicUiAction};
use crate::app::presentation::{
    ACCENT, GAME_SIDE_CONTROL_SCALE, GameButtonImageMode, GameButtonSpec, MUTED, TEXT, add_text,
    add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{NavigationUiAction, UiAction, add_cozy_panel};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{DraggableWindow, UiComposer, WindowDragHandle, spawn_node};
use std::marker::PhantomData;

pub(crate) fn render_mini_player(
    commands: &mut UiComposer,
    root: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    if !state.show_in_game {
        return;
    }
    let scale = GAME_SIDE_CONTROL_SCALE;
    let anchor = commands.keyed_child(
        root,
        "mini-music",
        (
            Node {
                position_type: PositionType::Absolute,
                top: px(state.mini_window.position.y),
                left: px(state.mini_window.position.x),
                width: px(256.0 * scale),
                max_width: percent(45),
                ..default()
            },
            GlobalZIndex(950),
            DraggableWindow::<MusicState>(PhantomData),
            Pickable::IGNORE,
        ),
    );
    let panel = add_cozy_panel(
        commands,
        anchor,
        Node {
            width: percent(100),
            padding: UiRect::all(px(8.0 * scale)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0 * scale),
            ..default()
        },
        assets,
    );
    let heading = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            min_width: px(0),
            align_items: AlignItems::Center,
            column_gap: px(5.0 * scale),
            ..default()
        },
        None,
    );
    let handle = spawn_node(
        commands,
        heading,
        Node {
            min_width: px(0),
            min_height: px(28.0 * scale),
            flex_grow: 1.0,
            flex_basis: px(0),
            align_items: AlignItems::Center,
            column_gap: px(5.0 * scale),
            ..default()
        },
        None,
    );
    commands.entity(handle).insert((
        WindowDragHandle::<MusicState>(PhantomData),
        Pickable::default(),
    ));
    commands.child(
        handle,
        (
            Node {
                width: px(20.0 * scale),
                height: px(20.0 * scale),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(assets.home.music_icon.clone()),
            Pickable::IGNORE,
        ),
    );
    let title = state
        .playing
        .as_ref()
        .map_or("选择歌曲", |song| song.name.as_str());
    let title = add_text(commands, handle, title, 13.0 * scale, ACCENT, assets);
    commands.entity(title).insert((
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            overflow: Overflow::clip(),
            ..default()
        },
        TextLayout::default().with_no_wrap(),
        Pickable::IGNORE,
    ));
    mini_button(
        commands,
        heading,
        "歌单",
        UiAction::Navigation(NavigationUiAction::ToggleMusic),
        true,
        assets,
    );
    let controls = controls_row(commands, panel, scale);
    let queued = !state.queue.is_empty();
    for (label, action) in [
        ("上一曲", MusicUiAction::Previous),
        (
            if state.paused || state.playing.is_none() {
                "播放"
            } else {
                "暂停"
            },
            MusicUiAction::TogglePause,
        ),
        ("下一曲", MusicUiAction::Next),
    ] {
        mini_button(
            commands,
            controls,
            label,
            UiAction::Music(action),
            queued,
            assets,
        );
    }
    let controls = controls_row(commands, panel, scale);
    mini_button(
        commands,
        controls,
        "音量−",
        UiAction::Music(MusicUiAction::Volume(-10)),
        state.volume > 0.0,
        assets,
    );
    add_text(
        commands,
        controls,
        format!("{:.0}%", state.volume * 100.0),
        12.0 * scale,
        TEXT,
        assets,
    );
    mini_button(
        commands,
        controls,
        "音量+",
        UiAction::Music(MusicUiAction::Volume(10)),
        state.volume < 1.0,
        assets,
    );
    mini_button(
        commands,
        controls,
        if state.mode == PlayMode::Shuffle {
            "随机"
        } else {
            "顺序"
        },
        UiAction::Music(MusicUiAction::Mode),
        true,
        assets,
    );
    if state.audio_task.is_some() || state.error.is_some() {
        add_text(
            commands,
            panel,
            if state.audio_task.is_some() {
                "正在加载歌曲"
            } else {
                "播放遇到问题，请打开歌单查看"
            },
            11.0 * scale,
            MUTED,
            assets,
        );
    }
}

fn controls_row(commands: &mut UiComposer, parent: Entity, scale: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(4.0 * scale),
            row_gap: px(4.0 * scale),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        None,
    )
}

fn mini_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: UiAction,
    enabled: bool,
    assets: &UiAssets,
) {
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action: enabled.then_some(action),
            normal: &assets.home.button,
            hovered: &assets.home.purple_button_compact,
            width: label.chars().count() as f32 * 11.0 + 16.0,
            height: 28.0,
            font_size: 11.0,
            image_mode: GameButtonImageMode::Sliced {
                border: 32.0,
                corner_scale: 0.36,
            },
        }
        .scaled(GAME_SIDE_CONTROL_SCALE),
    );
}
