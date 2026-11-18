use super::assets::{DrawGuessAssets, DrawingSounds};
use super::client::DrawGuessClient;
use super::feedback::DrawingFeedback;
use super::state::{DrawTool, can_draw};
use crate::app::runtime::ClientResource;
use bevy::audio::{AudioSinkPlayback, GlobalVolume, Volume};
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MatchId};
use std::collections::VecDeque;
use std::mem;

#[derive(Clone, Copy)]
pub(super) enum GuessSound {
    Correct,
    Reveal,
}

#[derive(Default)]
pub(super) struct GuessSoundPlayback {
    key: Option<(MatchId, u32)>,
    pending: VecDeque<GuessSound>,
    cooldown: f32,
}

pub(super) fn play_guess_feedback(
    time: Res<Time>,
    client: Res<DrawGuessClient>,
    assets: Res<DrawGuessAssets>,
    mut feedback: ResMut<DrawingFeedback>,
    mut playback: Local<GuessSoundPlayback>,
    mut commands: Commands,
) {
    let cues = feedback.take_sounds();
    if playback.key != client.key {
        playback.key = client.key;
        playback.pending.clear();
        playback.cooldown = 0.0;
    }
    if client.key.is_none() {
        return;
    }
    playback.pending.extend(cues);
    playback.cooldown = (playback.cooldown - time.delta_secs()).max(0.0);
    if playback.cooldown > 0.0 {
        return;
    }
    let Some(sound) = playback.pending.pop_front() else {
        return;
    };
    // Keep successes distinct even after a delayed frame. Reveal follows the final success.
    playback.cooldown = 0.12;
    let (sound, volume) = match sound {
        GuessSound::Correct => (&assets.sounds.correct, 0.36),
        GuessSound::Reveal => (&assets.sounds.reveal, 0.48),
    };
    commands.spawn((
        AudioPlayer::new(sound.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawingSound {
    Draw,
    Erase,
    Fill,
}

impl DrawingSound {
    fn source(self, sounds: &DrawingSounds) -> Handle<AudioSource> {
        match self {
            Self::Draw => &sounds.draw,
            Self::Erase => &sounds.erase,
            Self::Fill => &sounds.fill,
        }
        .clone()
    }

    fn volume(self) -> f32 {
        match self {
            Self::Draw => 0.28,
            Self::Erase => 0.22,
            Self::Fill => 0.65,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct DrawingAudio {
    stroke: Option<DrawingSound>,
    fill: bool,
}

impl DrawingAudio {
    pub fn stroke(&mut self, tool: DrawTool) {
        self.stroke = match tool {
            DrawTool::Pen => Some(DrawingSound::Draw),
            DrawTool::AreaEraser | DrawTool::StrokeEraser => Some(DrawingSound::Erase),
            DrawTool::Fill => None,
        };
    }

    pub fn fill(&mut self) {
        self.fill = true;
    }
}

#[derive(Component)]
pub(super) struct DrawingPlayer {
    key: (MatchId, u32),
    sound: DrawingSound,
    level: f32,
}

pub(super) fn play(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<DrawGuessAssets>,
    global_volume: Res<GlobalVolume>,
    mut input: ResMut<DrawingAudio>,
    mut players: Query<(
        Entity,
        &mut DrawingPlayer,
        &mut PlaybackSettings,
        Option<&mut AudioSink>,
    )>,
    mut commands: Commands,
) {
    // Consume local input once. Snapshot echoes and UI rebuilds never enqueue sounds.
    let input = mem::take(&mut *input);
    let key = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::draw_guess)
        .filter(|game| can_draw(game))
        .map(|game| (game.match_id, game.state.turn));
    let mut continuing = false;
    for (entity, mut player, mut settings, sink) in &mut players {
        if key != Some(player.key) || (input.fill && player.sound == DrawingSound::Fill) {
            commands.entity(entity).despawn();
            continue;
        }
        if player.sound == DrawingSound::Fill {
            continue;
        }
        let active = input.stroke == Some(player.sound);
        continuing |= active;
        player.level = if active {
            (player.level + time.delta_secs() / 0.025).min(1.0)
        } else {
            (player.level - time.delta_secs() / 0.08).max(0.0)
        };
        if player.level <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            settings.volume = Volume::Linear(player.sound.volume() * player.level);
            if let Some(mut sink) = sink {
                sink.set_volume(settings.volume * global_volume.volume);
            }
        }
    }
    let Some(key) = key else { return };
    if !continuing && let Some(sound) = input.stroke {
        let level = (time.delta_secs() / 0.025).min(1.0);
        commands.spawn((
            DrawingPlayer { key, sound, level },
            AudioPlayer::new(sound.source(&assets.sounds)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(sound.volume() * level)),
        ));
    }
    if input.fill {
        let sound = DrawingSound::Fill;
        commands.spawn((
            DrawingPlayer {
                key,
                sound,
                level: 1.0,
            },
            AudioPlayer::new(sound.source(&assets.sounds)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sound.volume())),
        ));
    }
}
