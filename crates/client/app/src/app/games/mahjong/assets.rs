use super::{
    FAN_VOICE_SPECS, MahjongStatusImages, create_mahjong_status_images, mahjong_tile_asset_path,
    mahjong_tile_height_asset_path, mahjong_tile_voice_key,
};
use bevy::prelude::*;
use leocard_mahjong::{MahjongFan, MahjongTileKind, build_deck};
use leocard_protocol::PlayerGender;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub(super) ready_hand: Handle<Image>,
    pub(super) action_prompt: Handle<AudioSource>,
    fan_voices: BTreeMap<MahjongFan, [Handle<AudioSource>; 2]>,
    action_voices: [[[Handle<AudioSource>; 2]; 8]; 2],
    tile_voices: HashMap<MahjongTileKind, [[Handle<AudioSource>; 2]; 2]>,
    pub(super) status: MahjongStatusImages,
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
    pub(super) fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        let kinds = build_deck()
            .into_iter()
            .map(|tile| tile.kind())
            .collect::<HashSet<_>>();
        let tiles = kinds
            .iter()
            .copied()
            .map(|kind| (kind, asset_server.load(mahjong_tile_asset_path(kind))))
            .collect();
        let tile_heights = kinds
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    asset_server.load(mahjong_tile_height_asset_path(kind)),
                )
            })
            .collect();
        Self {
            tiles,
            tile_heights,
            tile_back: asset_server.load("cards/mahjong/hong-kong/back.png"),
            ready_hand: asset_server.load("ui/mahjong/ready-hand.png"),
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
            status: create_mahjong_status_images(images),
        }
    }

    pub(super) fn action_voice(
        &self,
        gender: PlayerGender,
        action: MahjongActionVoice,
    ) -> &Handle<AudioSource> {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        &self.action_voices[voice_set][action.index()][fastrand::usize(..2)]
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
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(MahjongAssets::load(&asset_server, &mut images));
}
