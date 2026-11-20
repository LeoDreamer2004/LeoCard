use super::actions::GuandanUiAction;
use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::presentation::{
    MUTED, PaperCardButtonTone, TableActionButton, TableButtonSkin, add_no_legal_response_hint,
    add_text, game_side_control_scale,
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
            min_height: px(54.0 * game_side_control_scale()),
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
        .scaled(game_side_control_scale())
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
            14.0 * game_side_control_scale(),
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

pub(super) fn render_hand_controls(
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
            height: px(34.0 * game_side_control_scale()),
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    if !can_control || game.result.is_some() {
        return;
    }
    let enabled = !ui.is_dealing()
        && game
            .players
            .iter()
            .any(|player| player.player.id == game.you && !player.auto_play);
    for (label, action, available) in [
        (
            "理牌",
            GuandanUiAction::GroupSelected,
            ui.arrangement
                .can_group(&ui.selected, &game.your_hand, game.level),
        ),
        (
            "还原",
            GuandanUiAction::ResetHandGroups,
            ui.arrangement.has_groups(),
        ),
    ] {
        TableActionButton {
            size: Vec2::new(88.0, 30.0),
            font_size: 16.0,
            ..TableActionButton::new(
                label,
                (enabled && available).then_some(UiAction::Guandan(action)),
                TableButtonSkin::PaperCard(PaperCardButtonTone::Stone),
            )
        }
        .scaled(game_side_control_scale())
        .mount(commands, row, assets);
    }
}
