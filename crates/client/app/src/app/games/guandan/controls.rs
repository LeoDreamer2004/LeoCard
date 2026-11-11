use super::actions::GuandanUiAction;
use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, MUTED, PaperCardButtonTone, TableActionButton, TableButtonSkin,
    add_no_legal_response_hint, add_text,
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
    can_control: bool,
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
    if !can_control || !local_turn(game) || ui.is_dealing() {
        return;
    }
    let mut button = |label: &str, action: Option<GuandanUiAction>, tone| {
        TableActionButton {
            size: Vec2::new(142.0, 48.0),
            ..TableActionButton::new(
                label,
                action.map(UiAction::Guandan),
                TableButtonSkin::PaperCard(tone),
            )
        }
        .scaled(GAME_SIDE_CONTROL_SCALE)
        .mount(commands, row, assets);
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
            PaperCardButtonTone::Pine,
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
            PaperCardButtonTone::Pine,
        );
        button(
            "不出",
            game.table.is_some().then_some(GuandanUiAction::Pass),
            PaperCardButtonTone::Stone,
        );
        button(
            "提示",
            (!ui.hints.is_empty()).then_some(GuandanUiAction::Hint),
            PaperCardButtonTone::Jade,
        );
        if game.table.is_some() && ui.hints.is_empty() {
            add_no_legal_response_hint(commands, row, "没有牌能压过桌面牌", assets);
        }
    }
}
