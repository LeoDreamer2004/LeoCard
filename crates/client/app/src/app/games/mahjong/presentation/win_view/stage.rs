use super::super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongWinEffectTier, mahjong_win_effect_tier,
    mahjong_win_stage_start,
};
use super::timeline::WinShowcaseTiming;
use super::{
    MahjongMajorFanAssets, MahjongWinArtMaterial, MahjongWinInkMaterial, MajorShowcaseContext,
    WinShowcaseContext, render_high_win_showcase, render_major_win_showcase,
};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};
use leocard_ui::UiComposer;

pub(super) struct WinStageContext<'a, 'w, 's> {
    pub commands: &'a mut UiComposer<'w, 's>,
    pub table: Entity,
    pub viewport: Entity,
    pub game: &'a MahjongSnapshot,
    pub result: &'a MahjongHandResultView,
    pub winner_index: usize,
    pub reveal_duration: f32,
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
    pub ink_materials: &'a mut Assets<MahjongWinInkMaterial>,
    pub major_assets: &'a MahjongMajorFanAssets,
    pub art_materials: &'a mut Assets<MahjongWinArtMaterial>,
}

pub(super) fn render_win_stage(context: WinStageContext<'_, '_, '_>) {
    let WinStageContext {
        commands,
        table,
        viewport,
        game,
        result,
        winner_index,
        reveal_duration,
        assets,
        game_assets,
        materials,
        ink_materials,
        major_assets,
        art_materials,
    } = context;
    let winner = &result.winners[winner_index];
    let tier = mahjong_win_effect_tier(winner);
    let showcase = WinShowcaseContext {
        commands,
        table,
        viewport,
        game,
        winner,
        assets,
        game_assets,
        materials,
        timing: WinShowcaseTiming {
            reveal_duration,
            start: mahjong_win_stage_start(result, winner_index),
            duration: tier.presentation_duration(),
        },
    };
    match tier {
        MahjongWinEffectTier::Normal => {}
        MahjongWinEffectTier::HighTotal => render_high_win_showcase(showcase, ink_materials),
        MahjongWinEffectTier::MajorFan => render_major_win_showcase(MajorShowcaseContext {
            showcase,
            assets: major_assets,
            materials: art_materials,
        }),
    }
}
