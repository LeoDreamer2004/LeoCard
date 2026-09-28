use super::{FAN_VOICE_SPECS, MahjongStatusImages, create_mahjong_status_images};
use bevy::prelude::*;
use leocard_mahjong::{Fan, MahjongTileKind, build_deck};
use leocard_protocol::PlayerGender;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub action_button: Handle<Image>,
    pub action_button_hover: Handle<Image>,
    pub action_pass: Handle<Image>,
    pub action_pass_hover: Handle<Image>,
    pub action_win: Handle<Image>,
    pub action_win_hover: Handle<Image>,
    pub fan_voices: BTreeMap<Fan, Handle<AudioSource>>,
    action_voices: Option<[[Handle<AudioSource>; 5]; 2]>,
    pub(super) status: MahjongStatusImages,
}

#[derive(Clone, Copy)]
pub(super) enum MahjongActionVoice {
    Chow,
    Pung,
    Kong,
    Win,
    SelfDraw,
}

impl MahjongActionVoice {
    const FILES: [&'static str; 5] = ["chow", "pung", "kong", "win", "self_draw"];

    const fn index(self) -> usize {
        match self {
            Self::Chow => 0,
            Self::Pung => 1,
            Self::Kong => 2,
            Self::Win => 3,
            Self::SelfDraw => 4,
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
            .map(|kind| {
                (
                    kind,
                    asset_server.load(crate::app::mahjong_tile_asset_path(kind)),
                )
            })
            .collect();
        let tile_heights = kinds
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    asset_server.load(crate::app::mahjong_tile_height_asset_path(kind)),
                )
            })
            .collect();
        Self {
            tiles,
            tile_heights,
            tile_back: asset_server.load("cards/mahjong/hong-kong/back.png"),
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
                        asset_server.load(format!("audio/mahjong/fans/{:?}.ogg", spec.fan)),
                    )
                })
                .collect(),
            action_voices: (cfg!(debug_assertions)
                || std::env::var_os("BEVY_ASSET_ROOT").is_some())
            .then(|| {
                ["male", "female"].map(|gender| {
                    MahjongActionVoice::FILES.map(|name| {
                        asset_server.load(format!("audio/mahjong/actions/{gender}/{name}.ogg"))
                    })
                })
            }),
            status: create_mahjong_status_images(images),
        }
    }

    pub(super) fn action_voice(
        &self,
        gender: PlayerGender,
        action: MahjongActionVoice,
    ) -> Option<&Handle<AudioSource>> {
        let voice_set = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        self.action_voices
            .as_ref()
            .map(|voices| &voices[voice_set][action.index()])
    }
}

pub(super) fn load_mahjong_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(MahjongAssets::load(&asset_server, &mut images));
}
