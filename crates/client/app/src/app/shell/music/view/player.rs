use super::super::state::MusicClock;
use super::super::{MusicState, MusicUiAction};
use super::widgets::{button, row};
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn render_player(
    commands: &mut UiComposer,
    parent: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    add_title(commands, parent, "播放器", 20.0, TEXT, assets);
    if let Some(song) = &state.playing {
        add_text(commands, parent, &song.name, 17.0, ACCENT, assets);
        add_text(commands, parent, song.artist_names(), 14.0, MUTED, assets);
    } else {
        add_text(
            commands,
            parent,
            "选择歌单中的歌曲开始播放",
            14.0,
            MUTED,
            assets,
        );
    }
    if state.audio_task.is_some() {
        add_text(commands, parent, "正在加载歌曲", 14.0, MUTED, assets);
    }
    let clock = add_text(commands, parent, "0:00 / 0:00", 14.0, MUTED, assets);
    commands.entity(clock).insert(MusicClock);
    let controls = row(commands, parent);
    let queued = !state.queue.is_empty();
    button(
        commands,
        controls,
        "上一首",
        MusicUiAction::Previous,
        queued,
        assets,
    );
    button(
        commands,
        controls,
        if state.paused || state.playing.is_none() {
            "播放"
        } else {
            "暂停"
        },
        MusicUiAction::TogglePause,
        queued,
        assets,
    );
    button(
        commands,
        controls,
        "下一首",
        MusicUiAction::Next,
        queued,
        assets,
    );
    button(
        commands,
        controls,
        "停止",
        MusicUiAction::Stop,
        state.playing.is_some(),
        assets,
    );
    button(
        commands,
        controls,
        state.mode.label(),
        MusicUiAction::Mode,
        true,
        assets,
    );
    let controls = row(commands, parent);
    button(
        commands,
        controls,
        "后退15秒",
        MusicUiAction::Seek(-15),
        state.audio_entity.is_some(),
        assets,
    );
    button(
        commands,
        controls,
        "前进15秒",
        MusicUiAction::Seek(15),
        state.audio_entity.is_some(),
        assets,
    );
    button(
        commands,
        controls,
        "音量−",
        MusicUiAction::Volume(-10),
        state.volume > 0.0,
        assets,
    );
    add_text(
        commands,
        controls,
        format!("{:.0}%", state.volume * 100.0),
        14.0,
        TEXT,
        assets,
    );
    button(
        commands,
        controls,
        "音量+",
        MusicUiAction::Volume(10),
        state.volume < 1.0,
        assets,
    );
    add_text(
        commands,
        parent,
        "关闭面板后继续播放，音乐音量同时受声音设置中的总音量控制。",
        13.0,
        MUTED,
        assets,
    );
}
