use super::super::{MusicState, MusicUiAction};
use super::widgets::{button, row};
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

const PLAYLIST_PAGE: usize = 5;
const SONG_PAGE: usize = 8;

pub(super) fn render_library(
    commands: &mut UiComposer,
    parent: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    add_title(commands, parent, "我的歌单", 20.0, TEXT, assets);
    if state.catalog_task.is_some() {
        add_text(commands, parent, "正在读取歌单", 14.0, MUTED, assets);
    } else if state.playlists.is_empty() {
        add_text(
            commands,
            parent,
            "账号暂无创建或收藏的歌单",
            14.0,
            MUTED,
            assets,
        );
    }
    for playlist in state
        .playlists
        .iter()
        .skip(state.playlist_page * PLAYLIST_PAGE)
        .take(PLAYLIST_PAGE)
    {
        let line = row(commands, parent);
        let selected = state.selected == Some(playlist.id);
        button(
            commands,
            line,
            if selected { "重载" } else { "打开" },
            MusicUiAction::Playlist(playlist.id),
            true,
            assets,
        );
        let owned = state
            .account
            .as_ref()
            .is_some_and(|account| account.id == playlist.creator.id);
        let text = format!(
            "{}  {}  {} 首",
            playlist.name,
            if owned { "创建" } else { "收藏" },
            playlist.track_count
        );
        let label = add_text(
            commands,
            line,
            text,
            15.0,
            if selected { ACCENT } else { TEXT },
            assets,
        );
        commands.entity(label).insert(Node {
            min_width: px(0),
            flex_basis: px(180),
            flex_grow: 1.0,
            ..default()
        });
    }
    pagination(
        commands,
        parent,
        state.playlist_page,
        state.playlists.len().div_ceil(PLAYLIST_PAGE),
        false,
        assets,
    );
    if state.selected.is_none() {
        return;
    }
    let title = state
        .playlists
        .iter()
        .find(|list| Some(list.id) == state.selected)
        .map_or("歌单曲目", |list| list.name.as_str());
    add_title(commands, parent, title, 20.0, TEXT, assets);
    if state.songs.is_empty() && state.catalog_task.is_none() {
        add_text(
            commands,
            parent,
            "歌单中暂无可读取的歌曲",
            14.0,
            MUTED,
            assets,
        );
    }
    for (index, song) in state
        .songs
        .iter()
        .enumerate()
        .skip(state.song_page * SONG_PAGE)
        .take(SONG_PAGE)
    {
        let line = row(commands, parent);
        let selected = state
            .playing
            .as_ref()
            .is_some_and(|playing| playing.id == song.id);
        button(
            commands,
            line,
            "播放",
            MusicUiAction::Song(index),
            true,
            assets,
        );
        let info = spawn_node(
            commands,
            line,
            Node {
                flex_direction: FlexDirection::Column,
                min_width: px(0),
                flex_basis: px(180),
                flex_grow: 1.0,
                row_gap: px(2),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            info,
            &song.name,
            15.0,
            if selected { ACCENT } else { TEXT },
            assets,
        );
        add_text(commands, info, song.artist_names(), 13.0, MUTED, assets);
    }
    pagination(
        commands,
        parent,
        state.song_page,
        state.songs.len().div_ceil(SONG_PAGE),
        true,
        assets,
    );
    add_text(
        commands,
        parent,
        "播放权限以网易云账号为准，受限歌曲可能只能试听或无法播放。",
        13.0,
        MUTED,
        assets,
    );
}

fn pagination(
    commands: &mut UiComposer,
    parent: Entity,
    page: usize,
    count: usize,
    songs: bool,
    assets: &UiAssets,
) {
    if count <= 1 {
        return;
    }
    let line = row(commands, parent);
    let action = |page| {
        if songs {
            MusicUiAction::SongPage(page)
        } else {
            MusicUiAction::PlaylistPage(page)
        }
    };
    button(
        commands,
        line,
        "上一页",
        action(page.saturating_sub(1)),
        page > 0,
        assets,
    );
    add_text(
        commands,
        line,
        format!("{} / {count}", page + 1),
        14.0,
        MUTED,
        assets,
    );
    button(
        commands,
        line,
        "下一页",
        action((page + 1).min(count - 1)),
        page + 1 < count,
        assets,
    );
}
