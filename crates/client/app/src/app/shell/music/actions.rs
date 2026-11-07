use super::playback::{play_index, stop_audio};
use super::state::{AccountResult, CatalogResult, MusicAudio, MusicState};
use super::task::Task;
use crate::app::shell::{DomainUiAction, PressedUiAction, UiAction};
use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::prelude::*;
use leocard_music::MusicClient;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub(crate) enum MusicUiAction {
    Login,
    Forget,
    Remember,
    Refresh,
    Playlist(u64),
    Song(usize),
    TogglePause,
    Stop,
    Next,
    Previous,
    Seek(i32),
    Volume(i8),
    Mode,
    ToggleInGame,
    PlaylistPage(usize),
    SongPage(usize),
}

impl DomainUiAction for MusicUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        if let UiAction::Music(action) = action {
            Some(action)
        } else {
            None
        }
    }
}

pub(super) fn dispatch_music_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut state: ResMut<MusicState>,
    mut commands: Commands,
    sinks: Query<&AudioSink, With<MusicAudio>>,
) {
    for action in actions.read() {
        let Some(action) = MusicUiAction::extract(&action.0) else {
            continue;
        };
        let result = handle(action, &mut state, &mut commands, &sinks);
        if let Err(error) = result {
            state.error = Some(error);
        }
    }
}

fn handle(
    action: &MusicUiAction,
    state: &mut MusicState,
    commands: &mut Commands,
    sinks: &Query<&AudioSink, With<MusicAudio>>,
) -> Result<(), String> {
    match *action {
        MusicUiAction::Login => {
            if state.account.is_some() {
                return Ok(());
            }
            state.error = None;
            state.qr_key = None;
            state.qr_image = None;
            state.status = "正在生成登录二维码".to_owned();
            state.account_task = Some(Task::start(|| {
                let client = MusicClient::new()?;
                let qr = client.qr_login()?;
                Ok(AccountResult::Qr(client, qr))
            })?);
        }
        MusicUiAction::Forget => {
            stop_audio(state, commands);
            state.account_task = None;
            state.catalog_task = None;
            state.client = None;
            state.account = None;
            state.saved = None;
            state.qr_key = None;
            state.qr_image = None;
            state.playlists.clear();
            state.selected = None;
            state.songs = Arc::default();
            state.queue = Arc::default();
            state.queue_index = None;
            state.playing = None;
            state.error = None;
            state.remember = false;
            state.status = "已解除关联".to_owned();
            state.save();
        }
        MusicUiAction::Remember => {
            state.remember = !state.remember;
            if !state.remember {
                state.saved = None;
            }
            state.save();
        }
        MusicUiAction::Refresh => {
            state.error = None;
            state.load_playlists()?;
        }
        MusicUiAction::Playlist(id) => {
            let client = state.client.clone().ok_or("请先关联网易云账号")?;
            state.error = None;
            state.selected = Some(id);
            state.song_page = 0;
            state.songs = Arc::default();
            state.catalog_task = Some(Task::start(move || {
                client
                    .songs(id)
                    .map(|songs| CatalogResult::Songs(id, songs))
            })?);
        }
        MusicUiAction::Song(index) => {
            state.queue = state.songs.clone();
            play_index(state, index, commands)?;
        }
        MusicUiAction::TogglePause => {
            if state.audio_entity.is_some() || state.audio_task.is_some() {
                state.paused = !state.paused;
            } else if let Some(index) = state.queue_index {
                play_index(state, index, commands)?;
            }
        }
        MusicUiAction::Stop => {
            stop_audio(state, commands);
            state.playing = None;
            state.paused = true;
        }
        MusicUiAction::Next => {
            if let Some(index) = state
                .queue_index
                .and_then(|index| state.mode.next(index, state.queue.len(), true))
            {
                play_index(state, index, commands)?;
            }
        }
        MusicUiAction::Previous => {
            if let Some(index) = state.queue_index.filter(|_| !state.queue.is_empty()) {
                play_index(
                    state,
                    (index + state.queue.len() - 1) % state.queue.len(),
                    commands,
                )?;
            }
        }
        MusicUiAction::Seek(seconds) => {
            if let Some(entity) = state.audio_entity
                && let Ok(sink) = sinks.get(entity)
            {
                let target = (sink.position().as_secs_f64() + seconds as f64)
                    .clamp(0.0, state.duration.as_secs_f64());
                sink.try_seek(Duration::from_secs_f64(target))
                    .map_err(|_| "这首歌曲暂不支持调整进度")?;
            }
        }
        MusicUiAction::Volume(change) => {
            state.volume = (state.volume + change as f32 / 100.0).clamp(0.0, 1.0);
            state.save();
        }
        MusicUiAction::Mode => {
            state.mode = state.mode.cycle();
            state.save();
        }
        MusicUiAction::ToggleInGame => {
            state.show_in_game = !state.show_in_game;
            state.save();
        }
        MusicUiAction::PlaylistPage(page) => state.playlist_page = page,
        MusicUiAction::SongPage(page) => state.song_page = page,
    }
    Ok(())
}
