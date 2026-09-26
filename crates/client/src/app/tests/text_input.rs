use super::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::window::Ime;
use bevy_clipboard::Clipboard;
use leocard_protocol::MAX_PLAYER_NAME_CHARS;

#[test]
fn saved_player_name_is_limited_by_unicode_characters() {
    let name = truncate_chars("一二三四五六七八", MAX_PLAYER_NAME_CHARS);
    assert_eq!(name, "一二三四五六七");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn ime_committed_chinese_is_accepted_by_the_player_name_filter() {
    let mut name = String::new();
    append_filtered_input(
        &mut name,
        InputField::PlayerName,
        "七鬼五二三玩家甲",
        MAX_PLAYER_NAME_CHARS,
    );

    assert_eq!(name, "七鬼五二三玩家");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn pasted_server_address_filters_whitespace_and_replaces_the_default_value() {
    let mut address = String::new();
    append_filtered_input(
        &mut address,
        InputField::JoinAddress,
        " 192.168.1.20:52300\n",
        64,
    );

    assert_eq!(address, "192.168.1.20:52300");

    let mut hostname = String::new();
    append_filtered_input(
        &mut hostname,
        InputField::JoinAddress,
        "frp-off.com:52436",
        64,
    );
    assert_eq!(hostname, "frp-off.com:52436");
}

#[test]
fn ctrl_a_replaces_the_entire_unicode_player_name_on_next_input() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .add_message::<Ime>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(Clipboard::default())
        .insert_resource(ConnectionDraft {
            player_name: "原来的名字".to_owned(),
            host_port: "52300".to_owned(),
            join_address: "127.0.0.1:52300".to_owned(),
            active: InputField::PlayerName,
            selected_all: false,
        })
        .insert_resource(PageErrorState::default())
        .insert_resource(ChatPanelState::default())
        .insert_resource(DeveloperHandInput::default())
        .insert_resource(UiState::default())
        .add_systems(Update, handle_text_input);

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::ControlLeft);
        keyboard.press(KeyCode::KeyA);
    }
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: KeyCode::KeyA,
            logical_key: Key::Character("a".into()),
            state: ButtonState::Pressed,
            text: Some("a".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    app.update();
    assert!(app.world().resource::<ConnectionDraft>().selected_all);

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.clear();
        keyboard.release(KeyCode::ControlLeft);
    }
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: KeyCode::KeyN,
            logical_key: Key::Character("新".into()),
            state: ButtonState::Pressed,
            text: Some("新".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    app.update();
    let form = app.world().resource::<ConnectionDraft>();
    assert_eq!(form.player_name, "新");
    assert!(!form.selected_all);
}
