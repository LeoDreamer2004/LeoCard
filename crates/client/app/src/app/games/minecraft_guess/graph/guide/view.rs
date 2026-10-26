use super::super::super::{MinecraftGuessAssets, MinecraftGuessUiAction};
use super::table::render_table;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, cozy_backdrop_color,
    cozy_panel_transform,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{TextureButton, UiComposer, spawn_node};

pub(super) fn render_guide(
    commands: &mut UiComposer,
    root: Entity,
    progress: f32,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(cozy_backdrop_color(progress)),
    );
    commands.entity(backdrop).insert((
        Pickable::default(),
        CozyModalBackdrop(CozyModalKind::MinecraftGraphGuide),
    ));
    let panel = spawn_node(
        commands,
        backdrop,
        Node {
            width: px(980),
            max_width: percent(92),
            height: percent(86),
            max_height: px(900),
            min_height: px(0),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert((
        game_assets.popup_frame(),
        GlobalZIndex(2201),
        Pickable::default(),
        CozyModalPanel(CozyModalKind::MinecraftGraphGuide),
        cozy_panel_transform(progress),
    ));
    let header = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            min_height: px(40),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_text(commands, header, "关系提示", 27.0, TEXT, assets);
    TextureButton {
        node: Node {
            width: px(36),
            height: px(36),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        normal: game_assets.button(true, false),
        hovered: game_assets.button(true, true),
        action: Some(UiAction::MinecraftGuess(
            MinecraftGuessUiAction::CloseGraphGuide,
        )),
        label: None,
        font_size: 18.0,
        color: TEXT,
        icon: Some((game_assets.close_icon(), 28.0)),
    }
    .mount(commands, header, &assets.font);
    let explanation = add_text(
        commands,
        panel,
        "未激活的连边只显示分组名称；两端节点均揭晓后，显示具体关系。方向箭头是否提前显示取决于房间设置。",
        16.0,
        MUTED,
        assets,
    );
    commands.entity(explanation).insert(Node {
        flex_shrink: 0.0,
        ..default()
    });
    render_table(commands, panel, assets, game_assets);
}
