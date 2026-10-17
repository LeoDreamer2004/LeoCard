use super::BlackjackPlayback;
use crate::app::runtime::{ClientResource, ServerNotification, UiAssets};
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_blackjack::{BlackjackAction, Phase};
use leocard_protocol::{BlackjackEvent, GameEvent, MatchId, PlayerId, ServerEvent};

#[derive(Resource)]
pub(super) struct BlackjackSounds {
    turn: Handle<AudioSource>,
}

#[derive(Resource, Default)]
pub(super) struct BlackjackFeedback {
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    turn: Option<(u8, Option<PlayerId>, Option<u8>, Phase)>,
}

pub(super) fn load_sounds(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(BlackjackSounds {
        turn: server.load("vendor/kenney/interface-sounds/Audio/pluck_001.ogg"),
    });
}

pub(super) fn play_feedback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    assets: Res<UiAssets>,
    playback: Res<BlackjackPlayback>,
    sounds: Res<BlackjackSounds>,
    mut feedback: ResMut<BlackjackFeedback>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().blackjack_game())
    else {
        notifications.clear();
        *feedback = BlackjackFeedback::default();
        return;
    };
    if feedback.match_id != Some(game.match_id) {
        *feedback = BlackjackFeedback {
            match_id: Some(game.match_id),
            ..default()
        };
    }
    let common = &assets.audio;
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::Blackjack(BlackjackEvent::ActionApplied {
            action,
            ..
        })) = notification.event
        else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if context.match_id != game.match_id
            || feedback
                .sequence
                .is_some_and(|sequence| sequence >= context.sequence)
        {
            continue;
        }
        feedback.sequence = Some(context.sequence);
        let variants = match action {
            BlackjackAction::Hit => continue,
            BlackjackAction::Bet { .. } | BlackjackAction::DoubleDown | BlackjackAction::Split => {
                &common.shove_sounds
            }
            BlackjackAction::Stand => &common.place_sounds,
        };
        if let Some(sound) = variants.get(fastrand::usize(..variants.len().max(1))) {
            play(&mut commands, sound.clone(), 0.55);
        }
    }
    let turn = Some((
        game.round,
        game.current_player,
        game.current_hand,
        game.phase,
    ));
    if playback.busy() {
        return;
    }
    if feedback.turn != turn && game.current_player == Some(game.you) {
        play(&mut commands, sounds.turn.clone(), 0.55);
    }
    feedback.turn = turn;
}

fn play(commands: &mut Commands, sound: Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            ..PlaybackSettings::DESPAWN
        },
    ));
}
