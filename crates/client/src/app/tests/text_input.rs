use super::prelude::*;
use bevy::{
    input::{
        ButtonState, InputPlugin as NativeInputPlugin,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{InputDispatchPlugin, InputFocus, InputFocusPlugin},
    picking::events::{Pointer, Release},
    text::{
        EditableText, Font, FontCx, LayoutCx, apply_text_edits,
        load_font_assets_into_font_collection,
    },
    ui_widgets::EditableTextInputPlugin,
    window::{Ime, PrimaryWindow, WindowPlugin},
};
use bevy_clipboard::Clipboard;
use leocard_protocol::MAX_PLAYER_NAME_CHARS;
use std::{fs, path::Path};

#[test]
fn saved_player_name_is_limited_by_unicode_characters() {
    let name = truncate_chars("一二三四五六七八", MAX_PLAYER_NAME_CHARS);
    assert_eq!(name, "一二三四五六七");
}

fn input_app(value: &str) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((
        NativeInputPlugin,
        InputFocusPlugin,
        InputDispatchPlugin,
        WindowPlugin {
            primary_window: None,
            ..default()
        },
        EditableTextInputPlugin,
    ))
    .init_resource::<FontCx>()
    .init_resource::<Assets<Font>>()
    .init_resource::<LayoutCx>()
    .init_resource::<UiScale>()
    .insert_resource(Clipboard::default())
    .add_message::<Pointer<Release>>()
    .add_systems(
        PostUpdate,
        (load_font_assets_into_font_collection, apply_text_edits).chain(),
    );
    let font = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/ChillRoundGothic-Medium.ttf"),
    )
    .unwrap();
    let font = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(font));
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.update();
    let alias = app
        .world()
        .resource::<Assets<Font>>()
        .get(&font)
        .unwrap()
        .alias
        .clone();
    app.world_mut()
        .resource_mut::<FontCx>()
        .set_sans_serif_family(&alias)
        .unwrap();
    let editor = app.world_mut().spawn(EditableText::new(value)).id();
    app.insert_resource(InputFocus::from_entity(editor));
    app.update();
    (app, window, editor)
}

fn press(app: &mut App, window: Entity, key_code: KeyCode, key: Key, text: Option<&str>) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: key,
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window,
    });
}

#[test]
fn native_shortcut_replaces_the_selected_unicode_text() {
    let (mut app, window, editor) = input_app("原来的名字");
    press(&mut app, window, KeyCode::ControlLeft, Key::Control, None);
    press(
        &mut app,
        window,
        KeyCode::KeyA,
        Key::Character("a".into()),
        Some("a"),
    );
    app.update();
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::ControlLeft,
        logical_key: Key::Control,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    });
    press(
        &mut app,
        window,
        KeyCode::KeyN,
        Key::Character("新".into()),
        Some("新"),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(editor)
            .unwrap()
            .value()
            .to_string(),
        "新"
    );
}

#[test]
fn native_ime_keeps_preedit_out_of_the_committed_value() {
    let (mut app, window, editor) = input_app("玩家");
    app.world_mut().write_message(Ime::Preedit {
        window,
        value: "ni".to_owned(),
        cursor: Some((2, 2)),
    });
    app.update();
    let input = app.world().get::<EditableText>(editor).unwrap();
    assert!(input.is_composing());
    assert_eq!(input.value().to_string(), "玩家");
    app.world_mut().write_message(Ime::Commit {
        window,
        value: "你".to_owned(),
    });
    app.update();
    let input = app.world().get::<EditableText>(editor).unwrap();
    assert!(!input.is_composing());
    assert_eq!(input.value().to_string(), "玩家你");
}
