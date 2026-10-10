use super::{
    FAN_VOICE_SPECS, MahjongStatusImages, create_mahjong_status_images, mahjong_tile_asset_path,
    mahjong_tile_height_asset_path,
};
use bevy::prelude::*;
use leocard_mahjong::{Fan, MahjongTileKind, build_deck};
use leocard_protocol::PlayerGender;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub(super) ready_hand: Handle<Image>,
    pub(super) action_prompt: Handle<AudioSource>,
    pub action_button: Handle<Image>,
    pub action_button_hover: Handle<Image>,
    pub action_pass: Handle<Image>,
    pub action_pass_hover: Handle<Image>,
    pub action_win: Handle<Image>,
    pub action_win_hover: Handle<Image>,
    fan_voices: BTreeMap<Fan, [Handle<AudioSource>; 2]>,
    action_voices: [[[Handle<AudioSource>; 2]; 8]; 2],
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
            action_button: asset_server.load("ui/qigui523/play-normal.png"),
            action_button_hover: asset_server.load("ui/qigui523/play-hover.png"),
            action_pass: asset_server.load("ui/qigui523/pass-normal.png"),
            action_pass_hover: asset_server.load("ui/qigui523/pass-hover.png"),
            action_win: asset_server.load("ui/qigui523/warning-normal.png"),
            action_win_hover: asset_server.load("ui/qigui523/warning-hover.png"),
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

    pub(super) fn fan_voice(&self, gender: PlayerGender, fan: Fan) -> Option<&Handle<AudioSource>> {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        self.fan_voices.get(&fan).map(|voices| &voices[voice_set])
    }
}

pub(super) fn load_mahjong_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(MahjongAssets::load(&asset_server, &mut images));
}
