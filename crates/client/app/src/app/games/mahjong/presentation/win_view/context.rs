use super::timeline::WinShowcaseTiming;
use crate::app::games::mahjong::{MahjongAssets, MahjongTileMaterial};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_mahjong::MahjongWinRecord;
use leocard_protocol::MahjongSnapshot;
use leocard_ui::UiComposer;

pub(super) struct WinShowcaseContext<'a, 'w, 's> {
    pub commands: &'a mut UiComposer<'w, 's>,
    pub table: Entity,
    pub viewport: Entity,
    pub game: &'a MahjongSnapshot,
    pub winner: &'a MahjongWinRecord,
    pub timing: WinShowcaseTiming,
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}
