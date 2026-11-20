use super::actions::DrawGuessUiAction;
use super::assets::DrawGuessAssets;
use super::palette::{self, ink_color};
use super::state::{DrawGuessUiState, DrawTool};
use super::widgets::{ButtonKind, ButtonSpec, button};
use crate::app::presentation::game_side_control_scale;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_draw_guess::{DrawAction, Ink};
use leocard_platform::is_mobile;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    ui: &DrawGuessUiState,
    assets: &UiAssets,
    art: &DrawGuessAssets,
) {
    let scale = game_side_control_scale();
    let panel = spawn_node(
        commands,
        parent,
        Node {
            width: px(110.0 * scale),
            min_height: px(0),
            height: percent(100),
            flex_shrink: 0.0,
            padding: UiRect::all(px(6.0 * scale)),
            row_gap: px(5.0 * scale),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert(art.skin.inset());
    if is_mobile() {
        tools(commands, panel, assets, art);
        history_actions(commands, panel, assets, art);
    }
    palette::render(commands, panel, ui);
    presets(commands, panel, assets, art);
    if !is_mobile() {
        tools(commands, panel, assets, art);
    }
    sizes(commands, panel, assets, art);
    if !is_mobile() {
        history_actions(commands, panel, assets, art);
    }
    let mut clear = ButtonSpec::new(
        "清空",
        Some(UiAction::DrawGuess(DrawGuessUiAction::Draw(
            DrawAction::Clear,
        ))),
        percent(100),
    );
    clear.icon = Some(art.icons.clear.clone());
    clear.height = 40.0;
    button(commands, panel, clear, assets, art);
}

fn row(commands: &mut UiComposer, parent: Entity, wrap: bool) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            column_gap: px(3.0 * game_side_control_scale()),
            row_gap: px(3.0 * game_side_control_scale()),
            flex_wrap: if wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            ..default()
        },
        None,
    )
}

fn presets(commands: &mut UiComposer, parent: Entity, assets: &UiAssets, art: &DrawGuessAssets) {
    let row = row(commands, parent, true);
    for color in [
        Ink::BLACK,
        Ink::WHITE,
        Ink([232, 55, 55]),
        Ink([245, 145, 40]),
        Ink([250, 215, 45]),
        Ink([50, 172, 95]),
        Ink([55, 115, 225]),
        Ink([160, 85, 210]),
    ] {
        let mut spec = ButtonSpec::new(
            "",
            Some(UiAction::DrawGuess(DrawGuessUiAction::Color(color))),
            px(21),
        );
        spec.height = 21.0;
        spec.kind = ButtonKind::Color(color);
        let button = button(commands, row, spec, assets, art);
        let inset = px(4.0 * game_side_control_scale());
        commands.child(
            button,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: inset,
                    right: inset,
                    top: inset,
                    bottom: inset,
                    ..default()
                },
                BackgroundColor(ink_color(color)),
                Pickable::IGNORE,
            ),
        );
    }
}

fn tools(commands: &mut UiComposer, parent: Entity, assets: &UiAssets, art: &DrawGuessAssets) {
    let row = row(commands, parent, true);
    for (label, tool) in [
        ("画笔", DrawTool::Pen),
        ("橡皮", DrawTool::AreaEraser),
        ("整笔", DrawTool::StrokeEraser),
        ("填充", DrawTool::Fill),
    ] {
        let mut spec = ButtonSpec::new(
            label,
            Some(UiAction::DrawGuess(DrawGuessUiAction::Tool(tool))),
            percent(48),
        );
        spec.kind = ButtonKind::Tool(tool);
        spec.height = 44.0;
        spec.icon = Some(art.icons.tool(tool));
        button(commands, row, spec, assets, art);
    }
}

fn sizes(commands: &mut UiComposer, parent: Entity, assets: &UiAssets, art: &DrawGuessAssets) {
    let row = row(commands, parent, false);
    for size in [2, 6, 14, 30] {
        let label = size.to_string();
        let mut spec = ButtonSpec::new(
            &label,
            Some(UiAction::DrawGuess(DrawGuessUiAction::Size(size))),
            px(22),
        );
        spec.kind = ButtonKind::Size(size);
        spec.height = 27.0;
        button(commands, row, spec, assets, art);
    }
}

fn history_actions(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    art: &DrawGuessAssets,
) {
    let row = row(commands, parent, false);
    for (label, action, kind, icon) in [
        (
            "撤销",
            DrawAction::Undo,
            ButtonKind::Undo,
            art.icons.undo.clone(),
        ),
        (
            "重做",
            DrawAction::Redo,
            ButtonKind::Redo,
            art.icons.redo.clone(),
        ),
    ] {
        let mut spec = ButtonSpec::new(
            if is_mobile() { "" } else { label },
            Some(UiAction::DrawGuess(DrawGuessUiAction::Draw(action))),
            percent(48),
        );
        spec.kind = kind;
        spec.icon = Some(icon);
        spec.height = if is_mobile() { 32.0 } else { 40.0 };
        button(commands, row, spec, assets, art);
    }
}
