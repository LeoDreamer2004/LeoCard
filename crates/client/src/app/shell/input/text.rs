//! 文本输入、聊天输入与开发者手牌语法。

use crate::app::{
    ChatPanelState, ClientResource, ConnectionDraft, DeveloperHandInput, InputField,
    PageErrorState, UiState, append_developer_hand_input,
};
#[cfg(feature = "developer")]
use crate::app::submit_developer_hand;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::window::Ime;
use bevy_clipboard::Clipboard;
use leocard_protocol::{ChatContent, ClientCommand, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS};

pub(crate) fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut ime_inputs: MessageReader<Ime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<Clipboard>,
    mut form: ResMut<ConnectionDraft>,
    mut page_error: ResMut<PageErrorState>,
    mut chat: ResMut<ChatPanelState>,
    mut developer_hand: ResMut<DeveloperHandInput>,
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<UiState>,
) {
    for input in ime_inputs.read() {
        let Ime::Commit { value, .. } = input else {
            continue;
        };
        if chat.focused {
            append_chat_input(&mut chat.input, value);
        } else if !developer_hand.focused && form.active == InputField::PlayerName {
            append_filtered_input(
                &mut form.player_name,
                InputField::PlayerName,
                value,
                MAX_PLAYER_NAME_CHARS,
            );
            page_error.error = None;
            ui.dirty = true;
        }
    }

    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if control {
        if keyboard.just_pressed(KeyCode::KeyV) && developer_hand.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_developer_hand_input(&mut developer_hand.value, &text);
            }
        } else if keyboard.just_pressed(KeyCode::KeyV) && chat.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_chat_input(&mut chat.input, &text);
            }
        } else if form.active == InputField::JoinAddress && keyboard.just_pressed(KeyCode::KeyV) {
            let mut read = clipboard.fetch_text();
            match read.poll_result() {
                Some(Ok(text)) => {
                    let mut address = String::new();
                    append_filtered_input(&mut address, InputField::JoinAddress, &text, 64);
                    if address.is_empty() {
                        page_error.error = Some("剪贴板中没有可用的服务器地址".to_owned());
                    } else {
                        form.join_address = address;
                        page_error.error = None;
                    }
                }
                Some(Err(error)) => {
                    page_error.error = Some(format!("无法读取剪贴板：{error}"));
                }
                None => {
                    page_error.error = Some("剪贴板内容尚未准备好".to_owned());
                }
            }
            ui.dirty = true;
        }
        return;
    }
    for input in keyboard_inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        match input.logical_key {
            Key::Backspace => {
                if developer_hand.focused {
                    developer_hand.value.pop();
                    page_error.error = None;
                    continue;
                }
                if chat.focused {
                    chat.input.pop();
                    continue;
                }
                active_input_mut(&mut form).pop();
                page_error.error = None;
                ui.dirty = true;
            }
            Key::Tab => {
                if chat.focused || developer_hand.focused {
                    continue;
                }
                form.active = match form.active {
                    InputField::PlayerName => InputField::HostPort,
                    InputField::HostPort => InputField::JoinAddress,
                    InputField::JoinAddress => InputField::PlayerName,
                };
                ui.dirty = true;
            }
            Key::Enter if chat.focused => {
                let message = chat.input.trim();
                if !message.is_empty()
                    && let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Chat {
                        content: ChatContent::Text(message.to_owned()),
                    })
                {
                    chat.input.clear();
                }
            }
            Key::Enter if developer_hand.focused => {
                developer_hand.focused = false;
                #[cfg(feature = "developer")]
                submit_developer_hand(
                    &mut developer_hand,
                    &mut page_error,
                    client.as_deref_mut(),
                );
            }
            Key::Escape if chat.focused => {
                chat.focused = false;
                chat.quick_voice_open = false;
            }
            Key::Escape if developer_hand.focused => {
                developer_hand.focused = false;
            }
            _ => {
                let Some(text) = input.text.as_deref() else {
                    continue;
                };
                if chat.focused {
                    append_chat_input(&mut chat.input, text);
                    continue;
                }
                if developer_hand.focused {
                    append_developer_hand_input(&mut developer_hand.value, text);
                    page_error.error = None;
                    continue;
                }
                let active = form.active;
                let value = active_input_mut(&mut form);
                let maximum = match active {
                    InputField::PlayerName => MAX_PLAYER_NAME_CHARS,
                    InputField::HostPort => 5,
                    InputField::JoinAddress => 64,
                };
                append_filtered_input(value, active, text, maximum);
                page_error.error = None;
                ui.dirty = true;
            }
        }
    }
}

pub(crate) fn append_chat_input(value: &mut String, text: &str) {
    for character in text.chars().filter(|character| !character.is_control()) {
        if value.chars().count() >= MAX_CHAT_MESSAGE_CHARS {
            break;
        }
        value.push(character);
    }
}

pub(crate) fn append_filtered_input(
    value: &mut String,
    field: InputField,
    text: &str,
    maximum: usize,
) {
    for character in text.chars().filter(|character| !character.is_control()) {
        let allowed = match field {
            InputField::PlayerName => true,
            InputField::HostPort => character.is_ascii_digit(),
            InputField::JoinAddress => {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | ':' | '[' | ']' | '-')
            }
        };
        if allowed && value.chars().count() < maximum {
            value.push(character);
        }
    }
}

fn active_input_mut(form: &mut ConnectionDraft) -> &mut String {
    match form.active {
        InputField::PlayerName => &mut form.player_name,
        InputField::HostPort => &mut form.host_port,
        InputField::JoinAddress => &mut form.join_address,
    }
}
