use super::BlackjackPlayback;
use super::playback::SETTLEMENT_CHIP_DURATION;
use crate::app::presentation::{PokerChipSoundKind, PokerChipSounds, numbered_table_sounds};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_blackjack::{BlackjackAction, Phase};
use leocard_protocol::{BlackjackEvent, GameEvent, MatchId, PlayerId, ServerEvent};

#[derive(Resource)]
pub(super) struct BlackjackSounds {
    turn: Handle<AudioSource>,
    stand: Vec<Handle<AudioSource>>,
    chips: PokerChipSounds,
}

#[derive(Resource, Default)]
pub(super) struct BlackjackFeedback {
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    turn: Option<(u8, Option<PlayerId>, Option<u8>, Phase)>,
    settled_round: Option<u8>,
    chips: Vec<ChipCue>,
}

struct ChipCue {
    kind: PokerChipSoundKind,
    remaining: f32,
    volume: f32,
}

impl BlackjackFeedback {
    fn queue_chip(&mut self, kind: PokerChipSoundKind, remaining: f32, volume: f32) {
        self.chips.push(ChipCue {
            kind,
            remaining,
            volume,
        });
    }
}

pub(super) fn load_sounds(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(BlackjackSounds {
        turn: server.load("vendor/kenney/interface-sounds/Audio/pluck_001.ogg"),
        stand: numbered_table_sounds(&server, "vendor/kenney/casino-audio/Audio", "card-place", 4),
        chips: PokerChipSounds::load(&server),
    });
}

pub(super) fn play_feedback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    playback: Res<BlackjackPlayback>,
    sounds: Res<BlackjackSounds>,
    mut feedback: ResMut<BlackjackFeedback>,
    time: Res<Time>,
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
        match action {
            BlackjackAction::Hit => {}
            BlackjackAction::Bet { .. } => {
                feedback.queue_chip(PokerChipSoundKind::Lay, 0.10, 0.52);
            }
            BlackjackAction::DoubleDown | BlackjackAction::Split => {
                feedback.queue_chip(PokerChipSoundKind::Handle, 0.0, 0.42);
                feedback.queue_chip(PokerChipSoundKind::Lay, 0.15, 0.54);
                feedback.queue_chip(PokerChipSoundKind::Collide, 0.28, 0.40);
            }
            BlackjackAction::Stand => {
                if let Some(sound) = sounds
                    .stand
                    .get(fastrand::usize(..sounds.stand.len().max(1)))
                {
                    play(&mut commands, sound.clone(), 0.55);
                }
            }
        }
    }
    if playback.settlement_started() && feedback.settled_round != Some(game.round) {
        feedback.settled_round = Some(game.round);
        feedback.queue_chip(PokerChipSoundKind::Handle, 0.02, 0.34);
        feedback.queue_chip(PokerChipSoundKind::Stack, SETTLEMENT_CHIP_DURATION, 0.54);
    }
    feedback.chips.retain_mut(|cue| {
        cue.remaining -= time.delta_secs();
        if cue.remaining > 0.0 {
            return true;
        }
        let variants = sounds.chips.variants(cue.kind);
        if let Some(sound) = variants.get(fastrand::usize(..variants.len().max(1))) {
            play(&mut commands, sound.clone(), cue.volume);
        }
        false
    });
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
