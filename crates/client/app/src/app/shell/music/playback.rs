use super::state::{MusicAudio, MusicClock, MusicState, PreparedAudio};
use super::task::Task;
use crate::app::shell::UiState;
use bevy::audio::{AudioSink, AudioSinkPlayback, GlobalVolume, Volume};
use bevy::prelude::*;
use rodio::{Decoder, Source};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

pub(super) fn stop_audio(state: &mut MusicState, commands: &mut Commands) {
    state.audio_task = None;
    if let Some(entity) = state.audio_entity.take() {
        commands.entity(entity).despawn();
    }
    state.audio_source = None;
}

pub(super) fn play_index(
    state: &mut MusicState,
    index: usize,
    commands: &mut Commands,
) -> Result<(), String> {
    let song = state
        .queue
        .get(index)
        .cloned()
        .ok_or("播放队列中没有这首歌")?;
    let client = state.client.clone().ok_or("请先关联网易云账号")?;
    stop_audio(state, commands);
    state.queue_index = Some(index);
    state.duration = Duration::from_millis(song.duration_ms);
    state.playing = Some(song.clone());
    state.paused = false;
    state.trial = false;
    state.error = None;
    state.audio_task = Some(Task::start(move || {
        let audio = client.audio(song.id)?;
        let bytes: Arc<[u8]> = audio.bytes.into();
        let decoder = Decoder::builder()
            .with_data(Cursor::new(bytes.clone()))
            .with_byte_len(bytes.len() as u64)
            .build()
            .map_err(|_| "歌曲音频格式暂不支持，请选择其他歌曲")?;
        let duration = decoder
            .total_duration()
            .unwrap_or(Duration::from_millis(song.duration_ms));
        Ok(PreparedAudio {
            bytes,
            duration,
            trial: audio.trial,
        })
    })?);
    Ok(())
}

pub(super) fn update_playback(
    mut commands: Commands,
    mut state: ResMut<MusicState>,
    mut sources: ResMut<Assets<AudioSource>>,
    mut sinks: Query<&mut AudioSink, With<MusicAudio>>,
    mut clocks: Query<&mut Text, With<MusicClock>>,
    mut ui: ResMut<UiState>,
    global_volume: Res<GlobalVolume>,
) {
    if let Some(result) = state.audio_task.as_ref().and_then(Task::poll) {
        state.audio_task = None;
        match result {
            Ok(audio) => {
                state.duration = audio.duration;
                state.trial = audio.trial;
                let source = sources.add(AudioSource { bytes: audio.bytes });
                let entity = commands
                    .spawn((
                        MusicAudio,
                        AudioPlayer::new(source.clone()),
                        PlaybackSettings {
                            volume: Volume::Linear(state.volume),
                            paused: state.paused,
                            ..default()
                        },
                    ))
                    .id();
                state.audio_source = Some(source);
                state.audio_entity = Some(entity);
            }
            Err(error) => {
                state.error = Some(error);
                state.paused = true;
            }
        }
        ui.dirty = true;
    }
    let mut finished = false;
    let mut position = Duration::ZERO;
    if let Some(entity) = state.audio_entity
        && let Ok(mut sink) = sinks.get_mut(entity)
    {
        sink.set_volume(Volume::Linear(state.volume) * global_volume.volume);
        if state.paused {
            sink.pause();
        } else {
            sink.play();
        }
        position = sink.position();
        finished = sink.empty() && !state.paused;
    }
    if finished {
        let next = state
            .queue_index
            .and_then(|index| state.mode.next(index, state.queue.len(), false));
        if let Some(index) = next {
            if let Err(error) = play_index(&mut state, index, &mut commands) {
                state.error = Some(error);
            }
        } else {
            stop_audio(&mut state, &mut commands);
            state.paused = true;
        }
        ui.dirty = true;
    }
    for mut text in &mut clocks {
        let label = format!(
            "{} / {}{}",
            clock(position),
            clock(state.duration),
            if state.trial { "  试听" } else { "" }
        );
        if text.0 != label {
            text.0 = label;
        }
    }
}

fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
