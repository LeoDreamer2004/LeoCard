use super::actions::{WordLibraryUiAction, action};
use super::state::{LibrarySelection, WordLibraryManager, WordLibraryRoom};
use super::{editor, list};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, add_title, page_frame_skin};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, CozyModalKind, CozyModalPanel, ShellOverlayLayer, add_centered_modal,
    add_cozy_button_variant_scaled, add_cozy_close_button, add_cozy_disabled_button_scaled,
    cozy_panel_transform,
};
use bevy::prelude::*;
use leocard_ui::{PageFrame, UiComposer, spawn_node};

pub(in super::super) struct LibraryView<'a> {
    pub manager: &'a WordLibraryManager,
    pub room: &'a WordLibraryRoom,
    pub host: bool,
    pub assets: &'a UiAssets,
}

pub(in super::super) fn render(commands: &mut UiComposer, root: Entity, view: LibraryView<'_>) {
    let manager = view.manager;
    if !manager.open && manager.progress == 0.0 {
        return;
    }
    let scale = GAME_SIDE_CONTROL_SCALE;
    let overlay = add_centered_modal(
        commands,
        root,
        ShellOverlayLayer::Dialog,
        CozyModalKind::DrawGuessLibraries,
        manager.progress,
        Some(action(WordLibraryUiAction::Close)),
    );
    let mut frame = PageFrame::new(page_frame_skin(view.assets));
    frame.node.width = px(665);
    frame.node.max_width = percent(94);
    frame.node.height = px(620);
    frame.node.max_height = percent(94);
    frame.heading.height = px(44.0 * scale);
    frame.content.padding = UiRect::all(px(14));
    frame.content.row_gap = px(12);
    frame.content.flex_basis = px(0);
    let frame = frame.mount(commands, overlay);
    let panel = frame.panel;
    commands.entity(panel).insert((
        CozyModalPanel(CozyModalKind::DrawGuessLibraries),
        cozy_panel_transform(manager.progress),
    ));
    let heading = frame.heading;
    add_title(
        commands,
        heading,
        "词库管理",
        24.0 * scale,
        Color::srgb(0.13, 0.14, 0.17),
        view.assets,
    );
    add_cozy_close_button(
        commands,
        heading,
        action(WordLibraryUiAction::Close),
        view.assets,
    );
    let body = spawn_node(
        commands,
        frame.content,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(16),
            ..default()
        },
        None,
    );
    list::render(commands, body, &view);
    editor::render(commands, body, &view);
}

pub(super) struct LibraryControl<'a> {
    pub label: &'a str,
    pub action: Option<WordLibraryUiAction>,
    pub variant: CozyButtonVariant,
    pub width: Val,
    pub grow: f32,
}

impl<'a> LibraryControl<'a> {
    pub fn new(label: &'a str, action: Option<WordLibraryUiAction>) -> Self {
        let variant = match action.as_ref() {
            Some(
                WordLibraryUiAction::Save | WordLibraryUiAction::Select(LibrarySelection::New),
            ) => CozyButtonVariant::Primary,
            Some(
                WordLibraryUiAction::Use
                | WordLibraryUiAction::Upload
                | WordLibraryUiAction::ReviewUpload(_),
            ) => CozyButtonVariant::Cool,
            Some(WordLibraryUiAction::Delete) => CozyButtonVariant::Danger,
            _ => CozyButtonVariant::Neutral,
        };
        Self {
            label,
            action,
            variant,
            width: percent(100),
            grow: 0.0,
        }
    }
}

pub(super) fn control(
    commands: &mut UiComposer,
    parent: Entity,
    spec: LibraryControl<'_>,
    assets: &UiAssets,
) {
    let scale = GAME_SIDE_CONTROL_SCALE;
    let button = if let Some(command) = spec.action {
        add_cozy_button_variant_scaled(
            commands,
            parent,
            spec.label,
            action(command),
            assets,
            spec.width,
            40.0 * scale,
            spec.variant,
            scale,
        )
    } else {
        add_cozy_disabled_button_scaled(
            commands,
            parent,
            spec.label,
            assets,
            spec.width,
            40.0 * scale,
            scale,
        )
    };
    commands.entity(button).insert(Node {
        width: spec.width,
        height: px(40.0 * scale),
        min_width: px(0),
        flex_shrink: 0.0,
        flex_grow: spec.grow,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        overflow: Overflow::clip(),
        ..default()
    });
}

pub(super) fn editable(manager: &WordLibraryManager) -> bool {
    manager.selection != LibrarySelection::Builtin
}
