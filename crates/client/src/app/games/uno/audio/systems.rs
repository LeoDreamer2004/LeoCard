use super::*;
use crate::app::games::uno::{UnoAssets, UnoHandCardButton};
use crate::app::runtime::ClientResource;
use bevy::audio::Volume;
use bevy::prelude::*;

type ChangedHandCardButtons<'w, 's> = Query<
    'w,
    's,
    &'static Interaction,
    (Changed<Interaction>, With<Button>, With<UnoHandCardButton>),
>;

pub(crate) fn play_uno_card_selection_sounds(
    buttons: ChangedHandCardButtons,
    assets: Res<UnoAssets>,
    mut commands: Commands,
) {
    if assets.sounds.select.is_empty() {
        return;
    }
    for interaction in &buttons {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let index = fastrand::usize(..assets.sounds.select.len());
        commands.spawn((
            AudioPlayer::new(assets.sounds.select[index].clone()),
            PlaybackSettings {
                volume: Volume::Linear(0.22),
                speed: 1.04,
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

pub(crate) fn play_uno_audio_cues(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UnoAssets>,
    mut state: ResMut<UnoAudioState>,
    mut commands: Commands,
) {
    if client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_none()
    {
        state.cues.clear();
        state.duck_remaining = 0.0;
        return;
    }

    state.duck_remaining = (state.duck_remaining - time.delta_secs()).max(0.0);
    let mut waiting = Vec::with_capacity(state.cues.len());
    let mut ready = Vec::new();
    for mut cue in std::mem::take(&mut state.cues) {
        cue.remaining -= time.delta_secs();
        if cue.remaining <= 0.0 {
            ready.push(cue);
        } else {
            waiting.push(cue);
        }
    }
    state.cues = waiting;

    if ready.iter().any(|cue| cue.kind.is_prominent()) {
        state.duck_remaining = 0.35;
    }
    for cue in ready {
        let variants = assets.sounds.variants(cue.kind);
        if variants.is_empty() {
            continue;
        }
        let volume = if cue.kind.is_card() && state.duck_remaining > 0.0 {
            cue.volume * 0.55
        } else {
            cue.volume
        };
        commands.spawn((
            AudioPlayer::new(variants[cue.seed as usize % variants.len()].clone()),
            PlaybackSettings {
                volume: Volume::Linear(volume),
                speed: cue.speed,
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}
