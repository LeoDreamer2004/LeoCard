use super::actions::GuandanUiAction;
use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, GameButtonSpec, GameButtonTone, MUTED, add_no_legal_response_hint,
    add_text, add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_guandan::{GuandanPhase, GuandanTributeAction, can_beat, classify};
use leocard_protocol::GuandanSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &GuandanSnapshot,
    ui: &GuandanUiState,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            min_height: px(54.0 * GAME_SIDE_CONTROL_SCALE),
            column_gap: px(10),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    if !local_turn(game) || ui.is_dealing() {
        return;
    }
    let mut button = |label: &str, action: Option<GuandanUiAction>, tone| {
        add_textured_game_button(
            commands,
            row,
            assets,
            GameButtonSpec::compact(assets, label, action.map(UiAction::Guandan), tone)
                .scaled(GAME_SIDE_CONTROL_SCALE),
        );
    };
    if matches!(game.phase, GuandanPhase::Tribute) {
        let action = game.tribute_action;
        let allowed = ui.selected.len() == 1
            && ui
                .selected
                .iter()
                .any(|card| tribute_choice(game, *card).is_some());
        button(
            if action == Some(GuandanTributeAction::ReturnTribute) {
                "还贡"
            } else {
                "进贡"
            },
            allowed.then_some(GuandanUiAction::SubmitTribute),
            GameButtonTone::Play,
        );
        add_text(
            commands,
            row,
            if action == Some(GuandanTributeAction::ReturnTribute) {
                "有 10 或以下须从中还牌 否则还最小牌"
            } else {
                "从高亮牌中选择进贡"
            },
            14.0 * GAME_SIDE_CONTROL_SCALE,
            MUTED,
            assets,
        );
    } else if matches!(game.phase, GuandanPhase::Playing) {
        let cards = ui.selected.iter().copied().collect::<Vec<_>>();
        let legal = classify(&cards, game.level).is_ok_and(|play| {
            game.table
                .as_ref()
                .is_none_or(|(_, previous)| can_beat(&play, previous, game.level))
        });
        button(
            "出牌",
            legal.then_some(GuandanUiAction::Play),
            GameButtonTone::Play,
        );
        button(
            "不出",
            game.table.is_some().then_some(GuandanUiAction::Pass),
            GameButtonTone::Pass,
        );
        button(
            "提示",
            (!ui.hints.is_empty()).then_some(GuandanUiAction::Hint),
            GameButtonTone::Hint,
        );
        if game.table.is_some() && ui.hints.is_empty() {
            add_no_legal_response_hint(commands, row, "没有牌能压过桌面牌", assets);
        }
    }
}
