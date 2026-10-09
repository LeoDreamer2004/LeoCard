use super::{ConfirmationDialog, ConfirmationUiAction};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, add_cozy_button,
    add_cozy_button_variant, add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::{prelude::*, ui::FocusPolicy};

pub(crate) fn render_confirmation(
    commands: &mut Commands,
    root: Entity,
    dialog: &ConfirmationDialog,
    assets: &UiAssets,
) {
    let Some(request) = &dialog.request else {
        return;
    };
    let overlay = spawn_node(
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
        Some(cozy_backdrop_color(dialog.progress)),
    );
    commands.entity(overlay).insert((
        GlobalZIndex(2400),
        FocusPolicy::Block,
        CozyModalBackdrop(CozyModalKind::Confirmation),
    ));
    let panel = add_cozy_panel(
        commands,
        overlay,
        Node {
            width: px(440),
            max_width: percent(90),
            padding: UiRect::all(px(26)),
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        CozyModalPanel(CozyModalKind::Confirmation),
        cozy_panel_transform(dialog.progress),
    ));
    add_text(commands, panel, &request.title, 25.0, TEXT, assets);
    add_text(commands, panel, &request.message, 18.0, MUTED, assets);
    let buttons = spawn_node(
        commands,
        panel,
        Node {
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    add_cozy_button(
        commands,
        buttons,
        "取消",
        UiAction::Confirmation(ConfirmationUiAction::Cancel),
        assets,
        px(130),
        44.0,
    );
    add_cozy_button_variant(
        commands,
        buttons,
        "确认",
        UiAction::Confirmation(ConfirmationUiAction::Accept),
        assets,
        px(130),
        44.0,
        CozyButtonVariant::Cool,
    );
}
