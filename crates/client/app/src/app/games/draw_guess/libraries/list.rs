use super::actions::WordLibraryUiAction;
use super::state::LibrarySelection;
use super::view::{LibraryControl, LibraryView, control};
use crate::app::presentation::game_side_control_scale;
use crate::app::shell::CozyButtonVariant;
use bevy::prelude::*;
use leocard_draw_guess::WordLibrary;
use leocard_ui::{Scrollbar, ScrollbarSkin, UiComposer, spawn_node};

pub(super) fn render(commands: &mut UiComposer, parent: Entity, view: &LibraryView<'_>) {
    let scale = game_side_control_scale();
    let panel = spawn_node(
        commands,
        parent,
        Node {
            width: px(230.0 * scale),
            max_width: percent(40),
            min_width: px(150),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            min_height: px(0),
            ..default()
        },
        None,
    );
    let row = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_basis: px(0),
            min_height: px(0),
            column_gap: px(4),
            ..default()
        },
        None,
    );
    let list = commands.child(
        row,
        (
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
        ),
    );
    let builtin = WordLibrary::builtin();
    for (selection, library) in [(LibrarySelection::Builtin, &builtin)].into_iter().chain(
        view.manager
            .store
            .entries
            .iter()
            .map(|entry| (LibrarySelection::Local(entry.id.clone()), &entry.library)),
    ) {
        let label = format!("{}  {}", library.title, library.words.len());
        let mut spec =
            LibraryControl::new(&label, Some(WordLibraryUiAction::Select(selection.clone())));
        spec.variant = if view.manager.selection == selection {
            CozyButtonVariant::Cool
        } else {
            CozyButtonVariant::Neutral
        };
        control(commands, list, spec, view.assets);
    }
    Scrollbar::spawn(
        commands,
        row,
        list,
        ScrollbarSkin::Colored(Color::srgb(0.62, 0.58, 0.76)),
    );
    for (label, command) in [
        (
            "新建词库",
            WordLibraryUiAction::Select(LibrarySelection::New),
        ),
        ("打开文件夹", WordLibraryUiAction::OpenDirectory),
        ("刷新词库", WordLibraryUiAction::Refresh),
    ] {
        control(
            commands,
            panel,
            LibraryControl::new(label, Some(command)),
            view.assets,
        );
    }
    if view.host
        && let Some(offer) = view.room.offers.first()
    {
        control(
            commands,
            panel,
            LibraryControl::new(
                "接收上传",
                Some(WordLibraryUiAction::ReviewUpload(offer.id)),
            ),
            view.assets,
        );
    }
}
