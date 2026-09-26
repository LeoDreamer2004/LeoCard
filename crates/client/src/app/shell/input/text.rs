//! 文本输入、聊天输入与开发者手牌语法。

#[cfg(feature = "developer")]
use crate::app::submit_developer_hand;
use crate::app::{
    ChatPanelState, ClientResource, ConnectionDraft, DeveloperHandInput, InputField,
    PageErrorState, UiState, append_developer_hand_input,
};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::window::Ime;
use bevy_clipboard::Clipboard;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{ChatContent, ClientCommand, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS};

#[derive(SystemParam)]
pub(crate) struct TextInputContext<'w, 's> {
    keyboard_inputs: MessageReader<'w, 's, KeyboardInput>,
    ime_inputs: MessageReader<'w, 's, Ime>,
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    clipboard: ResMut<'w, Clipboard>,
    form: ResMut<'w, ConnectionDraft>,
    page_error: ResMut<'w, PageErrorState>,
    chat: ResMut<'w, ChatPanelState>,
    developer_hand: ResMut<'w, DeveloperHandInput>,
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
}

pub(crate) fn handle_text_input(context: TextInputContext) {
    let TextInputContext {
        mut keyboard_inputs,
        mut ime_inputs,
        keyboard,
        mut clipboard,
        mut form,
        mut page_error,
        mut chat,
        mut developer_hand,
        mut client,
        mut ui,
    } = context;
    let mut form = &mut *form;
    let chat = &mut *chat;
    let developer_hand = &mut *developer_hand;
    let connection_visible = client.as_deref().is_none_or(|client| {
        matches!(
            client.0.model().phase(),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed
        )
    });
    for input in ime_inputs.read() {
        let Ime::Commit { value, .. } = input else {
            continue;
        };
        if chat.focused {
            clear_selection(&mut chat.input, &mut chat.selected_all);
            append_chat_input(&mut chat.input, value);
        } else if connection_visible
            && !developer_hand.focused
            && form.active == InputField::PlayerName
        {
            clear_selection(&mut form.player_name, &mut form.selected_all);
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
    if control && (developer_hand.focused || chat.focused || connection_visible) {
        if keyboard.just_pressed(KeyCode::KeyA) {
            if developer_hand.focused {
                developer_hand.selected_all = !developer_hand.value.is_empty();
            } else if chat.focused {
                chat.selected_all = !chat.input.is_empty();
            } else {
                form.selected_all = !active_input_mut(&mut form).is_empty();
                ui.dirty = true;
            }
        } else if keyboard.just_pressed(KeyCode::KeyC) || keyboard.just_pressed(KeyCode::KeyX) {
            let cut = keyboard.just_pressed(KeyCode::KeyX);
            let selected = if developer_hand.focused {
                developer_hand
                    .selected_all
                    .then(|| developer_hand.value.clone())
            } else if chat.focused {
                chat.selected_all.then(|| chat.input.clone())
            } else if connection_visible {
                form.selected_all
                    .then(|| active_input_mut(&mut form).clone())
            } else {
                None
            };
            if let Some(selected) = selected
                && !selected.is_empty()
            {
                match clipboard.set_text(selected) {
                    Ok(()) if cut => {
                        if developer_hand.focused {
                            clear_selection(
                                &mut developer_hand.value,
                                &mut developer_hand.selected_all,
                            );
                        } else if chat.focused {
                            clear_selection(&mut chat.input, &mut chat.selected_all);
                        } else {
                            let value = active_input_mut(&mut form);
                            value.clear();
                            form.selected_all = false;
                            ui.dirty = true;
                        }
                    }
                    Ok(()) => {}
                    Err(error) => page_error.error = Some(format!("无法写入剪贴板：{error}")),
                }
            }
        } else if keyboard.just_pressed(KeyCode::KeyV) {
            let mut read = clipboard.fetch_text();
            match read.poll_result() {
                Some(Ok(text)) if developer_hand.focused => {
                    clear_selection(&mut developer_hand.value, &mut developer_hand.selected_all);
                    append_developer_hand_input(&mut developer_hand.value, &text);
                    page_error.error = None;
                }
                Some(Ok(text)) if chat.focused => {
                    clear_selection(&mut chat.input, &mut chat.selected_all);
                    append_chat_input(&mut chat.input, &text);
                }
                Some(Ok(text)) => {
                    let active = form.active;
                    let mut pasted = String::new();
                    let maximum = maximum_length(active);
                    append_filtered_input(&mut pasted, active, &text, maximum);
                    if active == InputField::JoinAddress && pasted.is_empty() {
                        page_error.error = Some("剪贴板中没有可用的服务器地址".to_owned());
                    } else {
                        let selected_all = form.selected_all;
                        let value = active_input_mut(&mut form);
                        if selected_all || active == InputField::JoinAddress {
                            value.clear();
                        }
                        append_filtered_input(value, active, &pasted, maximum);
                        form.selected_all = false;
                        page_error.error = None;
                    }
                    ui.dirty = true;
                }
                Some(Err(error)) => page_error.error = Some(format!("无法读取剪贴板：{error}")),
                None => page_error.error = Some("剪贴板内容尚未准备好".to_owned()),
            }
        }
    }
    for input in keyboard_inputs.read() {
        if input.state != ButtonState::Pressed || control {
            continue;
        }
        match input.logical_key {
            Key::Backspace => {
                if developer_hand.focused {
                    if !clear_selection(&mut developer_hand.value, &mut developer_hand.selected_all)
                    {
                        developer_hand.value.pop();
                    }
                    page_error.error = None;
                    continue;
                }
                if chat.focused {
                    if !clear_selection(&mut chat.input, &mut chat.selected_all) {
                        chat.input.pop();
                    }
                    continue;
                }
                if !connection_visible {
                    continue;
                }
                let selected_all = form.selected_all;
                let value = active_input_mut(&mut form);
                if selected_all {
                    value.clear();
                } else {
                    value.pop();
                }
                form.selected_all = false;
                page_error.error = None;
                ui.dirty = true;
            }
            Key::Tab => {
                if chat.focused || developer_hand.focused {
                    continue;
                }
                if !connection_visible {
                    continue;
                }
                form.active = match form.active {
                    InputField::PlayerName => InputField::HostPort,
                    InputField::HostPort => InputField::JoinAddress,
                    InputField::JoinAddress => InputField::PlayerName,
                };
                form.selected_all = false;
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
                    chat.selected_all = false;
                }
            }
            Key::Enter if developer_hand.focused => {
                developer_hand.focused = false;
                developer_hand.selected_all = false;
                #[cfg(feature = "developer")]
                submit_developer_hand(developer_hand, &mut page_error, client.as_deref_mut());
            }
            Key::Escape if chat.focused => {
                chat.focused = false;
                chat.selected_all = false;
                chat.quick_voice_open = false;
            }
            Key::Escape if developer_hand.focused => {
                developer_hand.focused = false;
                developer_hand.selected_all = false;
            }
            _ => {
                let Some(text) = input.text.as_deref() else {
                    continue;
                };
                if chat.focused {
                    clear_selection(&mut chat.input, &mut chat.selected_all);
                    append_chat_input(&mut chat.input, text);
                    continue;
                }
                if developer_hand.focused {
                    clear_selection(&mut developer_hand.value, &mut developer_hand.selected_all);
                    append_developer_hand_input(&mut developer_hand.value, text);
                    page_error.error = None;
                    continue;
                }
                if !connection_visible {
                    continue;
                }
                let active = form.active;
                if form.selected_all {
                    active_input_mut(&mut form).clear();
                    form.selected_all = false;
                }
                let value = active_input_mut(&mut form);
                let maximum = maximum_length(active);
                append_filtered_input(value, active, text, maximum);
                page_error.error = None;
                ui.dirty = true;
            }
        }
    }
}

fn clear_selection(value: &mut String, selected_all: &mut bool) -> bool {
    if !*selected_all {
        return false;
    }
    value.clear();
    *selected_all = false;
    true
}

fn maximum_length(field: InputField) -> usize {
    match field {
        InputField::PlayerName => MAX_PLAYER_NAME_CHARS,
        InputField::HostPort => 5,
        InputField::JoinAddress => 64,
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
