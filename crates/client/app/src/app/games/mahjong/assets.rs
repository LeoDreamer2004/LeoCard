use super::{
    FAN_VOICE_SPECS, MahjongCalligraphyAssets, MahjongTurnLightMaterial, MahjongWindLabelMaterial,
    MahjongWindLabels, mahjong_tile_voice_key,
};
use crate::app::games::GameFeedback;
use crate::app::presentation::mahjong::MahjongTileAssets;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use leocard_mahjong::{MahjongFan, MahjongTileKind};
use leocard_protocol::PlayerGender;
use std::array;
use std::collections::{BTreeMap, HashMap};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub(super) visuals: MahjongTileAssets,
    pub(super) wind_labels: MahjongWindLabels,
    pub(super) ready_hand: Handle<Image>,
    pub(super) calligraphy: MahjongCalligraphyAssets,
    pub(super) action_prompt: Handle<AudioSource>,
    pub(super) dice_throw: Handle<AudioSource>,
    fan_voices: BTreeMap<MahjongFan, [Handle<AudioSource>; 2]>,
    action_voices: [[[Handle<AudioSource>; 2]; 8]; 2],
    tile_voices: HashMap<MahjongTileKind, [[Handle<AudioSource>; 2]; 2]>,
    pub(super) status: Handle<MahjongTurnLightMaterial>,
    pub(super) status_lights: [Handle<MahjongTurnLightMaterial>; 4],
}

#[derive(Clone, Copy)]
pub(super) enum MahjongActionVoice {
    Chow,
    Pung,
    MeldedKong,
    ConcealedKong,
    AddedKong,
    Win,
    SelfDraw,
    Flower,
}

impl MahjongActionVoice {
    const FILES: [&'static str; 8] = [
        "chow",
        "pung",
        "melded_kong",
        "concealed_kong",
        "added_kong",
        "win",
        "self_draw",
        "flower",
    ];

    const fn index(self) -> usize {
        match self {
            Self::Chow => 0,
            Self::Pung => 1,
            Self::MeldedKong => 2,
            Self::ConcealedKong => 3,
            Self::AddedKong => 4,
            Self::Win => 5,
            Self::SelfDraw => 6,
            Self::Flower => 7,
        }
    }
}

impl MahjongAssets {
    pub(super) fn load(
        asset_server: &AssetServer,
        visuals: MahjongTileAssets,
        lights: &mut Assets<MahjongTurnLightMaterial>,
        labels: &mut Assets<MahjongWindLabelMaterial>,
    ) -> Self {
        let load_image = |name| {
            asset_server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("ui/mahjong/{name}.png"))
        };
        let status_texture = load_image("center-panel");
        let mut panel = MahjongTurnLightMaterial::new(0, status_texture.clone());
        panel.params.w = 1.0;
        let status = lights.add(panel);
        Self {
            visuals,
            ready_hand: asset_server.load("ui/mahjong/ready-hand.png"),
            calligraphy: MahjongCalligraphyAssets::load(asset_server),
            fan_voices: FAN_VOICE_SPECS
                .iter()
                .map(|spec| {
                    (
                        spec.fan,
                        [
                            asset_server
                                .load(format!("audio/mahjong/fans/male/{:?}.ogg", spec.fan)),
                            asset_server
                                .load(format!("audio/mahjong/fans/female/{:?}.ogg", spec.fan)),
                        ],
                    )
                })
                .collect(),
            action_voices: ["male", "female"].map(|gender| {
                MahjongActionVoice::FILES.map(|name| {
                    [1, 2].map(|variant| {
                        asset_server.load(format!(
                            "audio/mahjong/actions/{gender}/{name}_{variant}.ogg"
                        ))
                    })
                })
            }),
            action_prompt: asset_server.load("vendor/kenney/interface-sounds/Audio/pluck_001.ogg"),
            dice_throw: asset_server.load("vendor/kenney/casino-audio/Audio/dice-throw-1.ogg"),
            tile_voices: (0..34)
                .filter_map(MahjongTileKind::from_index34)
                .map(|kind| {
                    let key =
                        mahjong_tile_voice_key(kind).expect("standard tiles have voice names");
                    let voices = ["male", "female"].map(|gender| {
                        [1, 2].map(|variant| {
                            asset_server
                                .load(format!("audio/mahjong/tiles/{gender}/{key}_{variant}.ogg"))
                        })
                    });
                    (kind, voices)
                })
                .collect(),
            wind_labels: MahjongWindLabels::load(asset_server, labels),
            status,
            status_lights: array::from_fn(|relative| {
                lights.add(MahjongTurnLightMaterial::new(
                    relative as u8,
                    status_texture.clone(),
                ))
            }),
        }
    }

    pub(super) fn fan_voice(
        &self,
        gender: PlayerGender,
        fan: MahjongFan,
    ) -> Option<&Handle<AudioSource>> {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        self.fan_voices.get(&fan).map(|voices| &voices[voice_set])
    }

    pub(super) fn tile_voice(
        &self,
        gender: PlayerGender,
        tile: MahjongTileKind,
    ) -> Option<&Handle<AudioSource>> {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        self.tile_voices
            .get(&tile)
            .map(|voices| &voices[voice_set][fastrand::usize(..2)])
    }
}

pub(super) fn load_mahjong_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    visuals: Res<MahjongTileAssets>,
    mut lights: ResMut<Assets<MahjongTurnLightMaterial>>,
    mut labels: ResMut<Assets<MahjongWindLabelMaterial>>,
) {
    commands.insert_resource(MahjongAssets::load(
        &asset_server,
        visuals.clone(),
        &mut lights,
        &mut labels,
    ));
}

impl<'a> GameFeedback<(&'a MahjongAssets, PlayerGender)> for MahjongActionVoice {
    type Cue = &'a Handle<AudioSource>;

    fn feedback(&self, (assets, gender): (&'a MahjongAssets, PlayerGender)) -> Self::Cue {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        &assets.action_voices[voice_set][self.index()][fastrand::usize(..2)]
    }
}
