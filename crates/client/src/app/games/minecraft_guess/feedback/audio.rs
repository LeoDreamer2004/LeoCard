use super::notes::{Instrument, Note};
use super::state::{FeedbackSound, GuessFeedback};
use crate::app::presentation::StartGameSeatTransition;
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::MatchId;
use std::collections::VecDeque;

#[derive(Resource)]
pub(in super::super) struct FeedbackAudio {
    guitar: Handle<AudioSource>,
    harp: Handle<AudioSource>,
    pling: Handle<AudioSource>,
    bass: Handle<AudioSource>,
    bell: Handle<AudioSource>,
}

impl FeedbackAudio {
    fn source(&self, instrument: Instrument) -> Handle<AudioSource> {
        match instrument {
            Instrument::Guitar => self.guitar.clone(),
            Instrument::Harp => self.harp.clone(),
            Instrument::Pling => self.pling.clone(),
            Instrument::Bass => self.bass.clone(),
            Instrument::Bell => self.bell.clone(),
        }
    }
}

struct ScheduledNote {
    at: f32,
    note: Note,
}

#[derive(Resource, Default)]
pub(in super::super) struct NotePlayback {
    key: Option<(MatchId, u8)>,
    pending: VecDeque<ScheduledNote>,
}

impl NotePlayback {
    fn enqueue(&mut self, at: f32, sound: FeedbackSound) {
        for &note in sound.notes() {
            let at = at + note.delay;
            let index = self.pending.partition_point(|pending| pending.at <= at);
            self.pending.insert(index, ScheduledNote { at, note });
        }
    }

    fn take_due(&mut self, clock: f32) -> Option<Note> {
        if self.pending.front()?.at > clock {
            return None;
        }
        self.pending.pop_front().map(|pending| pending.note)
    }
}

pub(in super::super) fn load_feedback_audio(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(FeedbackAudio {
        guitar: server.load("audio/minecraft_guess/note_block/guitar.ogg"),
        harp: server.load("audio/minecraft_guess/note_block/harp.ogg"),
        pling: server.load("audio/minecraft_guess/note_block/pling.ogg"),
        bass: server.load("audio/minecraft_guess/note_block/bass.ogg"),
        bell: server.load("audio/minecraft_guess/note_block/bell.ogg"),
    });
    commands.init_resource::<NotePlayback>();
}

pub(in super::super) fn play_feedback_audio(
    mut commands: Commands,
    mut playback: ResMut<GuessFeedback>,
    intro: Res<StartGameSeatTransition>,
    assets: Res<FeedbackAudio>,
    mut notes: ResMut<NotePlayback>,
) {
    if notes.key != playback.key {
        notes.key = playback.key;
        notes.pending.clear();
    }
    if playback.key.is_some_and(|(id, _)| intro.is_active_for(id)) {
        return;
    }
    while playback
        .cues
        .front()
        .is_some_and(|cue| cue.at <= playback.clock)
    {
        let cue = playback.cues.pop_front().expect("due audio cue");
        notes.enqueue(cue.at, cue.sound);
    }
    while let Some(note) = notes.take_due(playback.clock) {
        commands.spawn((
            AudioPlayer::new(assets.source(note.instrument)),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(note.volume))
                .with_speed(note.speed()),
        ));
    }
}
