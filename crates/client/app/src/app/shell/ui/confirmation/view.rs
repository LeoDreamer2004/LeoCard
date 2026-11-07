use super::{ConfirmationDialog, ConfirmationUiAction};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalKind, CozyModalPanel, ShellOverlayLayer, UiAction, add_centered_modal,
    add_cozy_button, add_cozy_button_variant, add_cozy_panel, cozy_panel_transform,
};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn render_confirmation(
    commands: &mut UiComposer,
    root: Entity,
    dialog: &ConfirmationDialog,
    assets: &UiAssets,
) {
    let Some(request) = &dialog.request else {
        return;
    };
    let cancel = UiAction::Confirmation(ConfirmationUiAction::Cancel);
    let overlay = add_centered_modal(
        commands,
        root,
        ShellOverlayLayer::Confirmation,
        CozyModalKind::Confirmation,
        dialog.progress,
        dialog.is_open().then(|| cancel.clone()),
    );
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
    add_title(commands, panel, &request.title, 25.0, TEXT, assets);
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
    if request.accept.is_some() {
        add_cozy_button(commands, buttons, "取消", cancel, assets, px(130), 44.0);
    }
    add_cozy_button_variant(
        commands,
        buttons,
        request.accept_label,
        UiAction::Confirmation(ConfirmationUiAction::Accept),
        assets,
        px(130),
        44.0,
        request.accept_variant,
    );
}
