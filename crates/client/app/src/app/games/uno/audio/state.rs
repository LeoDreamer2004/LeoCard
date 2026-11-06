use super::*;
use crate::app::games::GameFeedback;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::UnoEvent;

#[derive(Clone, Copy, Debug)]
pub(crate) struct UnoAudioCue {
    pub(super) kind: UnoSoundKind,
    pub(super) remaining: f32,
    pub(super) volume: f32,
    pub(super) speed: f32,
    pub(super) seed: u64,
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
        self.cues
            .extend(event.feedback((you, self.serial.wrapping_mul(37))));
    }
}
