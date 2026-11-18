use super::actions::WordLibraryUiAction;
use super::state::{LibraryField, LibrarySelection, LibraryWordCount};
use super::view::{LibraryControl, LibraryView, control, editable};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, MUTED, TEXT, add_text};
use crate::app::shell::ConfigurationScroll;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_draw_guess::WordLibrary;
use leocard_ui::{Scrollbar, ScrollbarSkin, TextInput, UiComposer, sliced_image, spawn_node};

pub(super) fn render(commands: &mut UiComposer, parent: Entity, view: &LibraryView<'_>) {
    let scale = GAME_SIDE_CONTROL_SCALE;
    let manager = view.manager;
    let panel = commands.keyed_child(
        parent,
        format!("library-editor:{}", manager.editor_revision),
        Node {
            min_width: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
    );
    let heading = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(40.0 * scale),
            min_width: px(0),
            flex_shrink: 0.0,
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    if editable(manager) {
        let mut title = TextInput::new("word-library-title", &manager.title);
        title.placeholder = "词库标题";
        title.max_characters = WordLibrary::MAX_TITLE_CHARACTERS;
        title.font_size = 17.0 * scale;
        let editor = title.spawn(
            commands,
            heading,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                height: px(40.0 * scale),
                flex_shrink: 0.0,
                padding: UiRect::all(px(8)),
                ..default()
            },
            view.assets.text_input_style(),
        );
        commands.entity(editor).insert(LibraryField::Title);
    } else {
        let title = add_text(
            commands,
            heading,
            "默认词库",
            18.0 * scale,
            TEXT,
            view.assets,
        );
        commands.entity(title).insert(Node {
            flex_grow: 1.0,
            min_width: px(0),
            ..default()
        });
    }
    let count = add_text(
        commands,
        heading,
        format!("{} 个词", manager.word_count()),
        15.0 * scale,
        MUTED,
        view.assets,
    );
    commands.entity(count).insert((
        LibraryWordCount,
        TextLayout::no_wrap(),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
    let body_row = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(4),
            ..default()
        },
        None,
    );
    let body = spawn_node(
        commands,
        body_row,
        Node {
            min_width: px(0),
            min_height: px(0),
            height: percent(100),
            flex_grow: 1.0,
            flex_basis: px(0),
            flex_direction: FlexDirection::Column,
            padding: if editable(manager) {
                UiRect::ZERO
            } else {
                UiRect::all(px(10))
            },
            overflow: if editable(manager) {
                Overflow::clip()
            } else {
                Overflow::scroll_y()
            },
            ..default()
        },
        None,
    );
    if editable(manager) {
        let mut content = TextInput::new("word-library-content", &manager.content);
        content.placeholder = "每行一个词";
        content.max_characters = WordLibrary::MAX_CONTENT_BYTES;
        content.filter = |character| character == '\n' || !character.is_control();
        content.multiline = true;
        content.font_size = 16.0 * scale;
        let editor = content.spawn(
            commands,
            body,
            Node {
                width: percent(100),
                height: percent(100),
                min_height: px(0),
                padding: UiRect::all(px(10)),
                ..default()
            },
            view.assets.text_input_style(),
        );
        commands.entity(editor).insert(LibraryField::Content);
    } else {
        commands.entity(body).insert((
            sliced_image(view.assets.home.input.clone(), 32.0, 0.55),
            ConfigurationScroll,
            RelativeCursorPosition::default(),
        ));
        commands.entity(body).init(ScrollPosition::default());
        let text = add_text(
            commands,
            body,
            &manager.content,
            16.0 * scale,
            TEXT,
            view.assets,
        );
        commands.entity(text).insert(Node {
            width: percent(100),
            flex_shrink: 0.0,
            ..default()
        });
        Scrollbar::spawn(
            commands,
            body_row,
            body,
            ScrollbarSkin::Colored(Color::srgb(0.62, 0.58, 0.76)),
        );
    }
    let row = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    let current = manager
        .selected_library()
        .is_some_and(|library| library.fingerprint() == view.room.selected.fingerprint);
    let local = matches!(manager.selection, LibrarySelection::Local(_));
    let commands_to_show = [
        (
            "保存",
            (editable(manager)).then_some(WordLibraryUiAction::Save),
        ),
        ("删除", local.then_some(WordLibraryUiAction::Delete)),
        (
            if view.host && current {
                "使用中"
            } else if view.host {
                "使用词库"
            } else {
                "上传给房主"
            },
            (!manager.dirty()
                && ((view.host && !current)
                    || (!view.host && local && view.room.offers.is_empty()))
                && manager.selection != LibrarySelection::New)
                .then_some(if view.host {
                    WordLibraryUiAction::Use
                } else {
                    WordLibraryUiAction::Upload
                }),
        ),
    ];
    for ((label, action), grow) in commands_to_show.into_iter().zip([1.0, 1.0, 2.0]) {
        let mut spec = LibraryControl::new(label, action);
        spec.width = px(0);
        spec.grow = grow;
        control(commands, row, spec, view.assets);
    }
}
