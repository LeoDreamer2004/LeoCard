use super::voices::{ShengjiVoiceCue, ShengjiVoiceKind};
use bevy::prelude::*;
use leocard_protocol::PlayerGender;

#[derive(Resource)]
pub(super) struct ShengjiVoiceAssets {
    clips: [[[Handle<AudioSource>; 3]; ShengjiVoiceKind::ALL.len()]; 2],
}

impl ShengjiVoiceAssets {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            clips: ["male", "female"].map(|gender| {
                ShengjiVoiceKind::ALL.map(|kind| {
                    [1, 2, 3].map(|variant| {
                        asset_server.load(format!(
                            "audio/shengji/voices/{gender}/{}_{variant}.ogg",
                            kind.asset_name()
                        ))
                    })
                })
            }),
        }
    }

    pub(super) fn random_clip(&self, cue: ShengjiVoiceCue) -> Handle<AudioSource> {
        let gender = match cue.gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        let variants = &self.clips[gender][cue.kind as usize];
        variants[fastrand::usize(..variants.len())].clone()
    }
}

pub(super) fn load_shengji_voice_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShengjiVoiceAssets::load(&asset_server));
}
