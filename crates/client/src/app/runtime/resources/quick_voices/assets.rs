use bevy::prelude::*;
use leocard_protocol::{PlayerGender, QUICK_VOICE_COUNT};
use std::array;

#[derive(Default)]
pub(crate) struct QuickVoiceAssets {
    male: [Handle<AudioSource>; QUICK_VOICE_COUNT as usize],
    female: [Handle<AudioSource>; QUICK_VOICE_COUNT as usize],
}

impl QuickVoiceAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let [male, female] = ["male", "female"].map(|gender| {
            array::from_fn(|index| server.load(format!("vendor/noname/voice/{gender}/{index}.mp3")))
        });
        Self { male, female }
    }

    pub(crate) fn get(&self, gender: PlayerGender, index: u8) -> Option<&Handle<AudioSource>> {
        let voices = match gender {
            PlayerGender::Male => &self.male,
            PlayerGender::Female => &self.female,
        };
        voices.get(usize::from(index))
    }
}
