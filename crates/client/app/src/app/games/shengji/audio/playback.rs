use super::voices::ShengjiVoiceState;
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::MatchId;

#[derive(Component)]
pub(super) struct ShengjiVoicePlayback {
    round: (MatchId, u32),
}

pub(super) fn play_shengji_voices(
    mut voices: ResMut<ShengjiVoiceState>,
    playing: Query<(Entity, &ShengjiVoicePlayback)>,
    mut commands: Commands,
) {
    let mut busy = false;
    for (entity, playback) in &playing {
        if voices.observed == Some(playback.round) {
            busy = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    // 等待实际音频实体播完并销毁，避免密集出牌时多段语音重叠。
    if !busy
        && let Some(round) = voices.observed
        && let Some(clip) = voices.pending.pop_front()
    {
        commands.spawn((
            ShengjiVoicePlayback { round },
            AudioPlayer::new(clip),
            PlaybackSettings {
                volume: Volume::Linear(0.88),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}
