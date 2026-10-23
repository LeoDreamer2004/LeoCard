use super::*;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, UnoEvent};

#[derive(Clone, Copy, Debug)]
pub(super) struct UnoAudioCue {
    pub kind: UnoSoundKind,
    pub remaining: f32,
    pub volume: f32,
    pub speed: f32,
    pub seed: u64,
}

impl UnoAudioCue {
    pub(super) fn new(kind: UnoSoundKind, remaining: f32, volume: f32, seed: u64) -> Self {
        Self {
            kind,
            remaining,
            volume,
            speed: 1.0,
            seed,
        }
    }

    pub(super) fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

#[derive(Resource, Default)]
pub(crate) struct UnoAudioState {
    pub(super) cues: Vec<UnoAudioCue>,
    pub(super) serial: u64,
    pub(super) duck_remaining: f32,
}

impl UnoAudioState {
    pub(crate) fn queue_event(&mut self, event: &UnoEvent, you: PlayerId) {
        self.serial = self.serial.wrapping_add(1);
        self.cues.extend(uno_event_sound_plan(
            event,
            you,
            self.serial.wrapping_mul(37),
        ));
    }
}
