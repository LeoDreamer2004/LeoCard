use super::{ChatPanelState, ChatTextInput};
use crate::app::presentation::{TextInputAction, TextInputEvent};
use crate::app::runtime::ClientResource;
use bevy::{
    input_focus::{FocusGained, InputFocus},
    prelude::*,
    text::EditableText,
};
use leocard_protocol::{ChatContent, ClientCommand};

pub(super) fn focus_chat_input(
    focus: On<FocusGained>,
    inputs: Query<(), With<ChatTextInput>>,
    mut chat: ResMut<ChatPanelState>,
) {
    if inputs.contains(focus.entity) {
        chat.open = true;
        chat.quick_voice_open = false;
        chat.emoji_open = false;
    }
}

pub(super) fn sync_chat_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<(Entity, &mut EditableText), With<ChatTextInput>>,
    mut chat: ResMut<ChatPanelState>,
    mut client: Option<ResMut<ClientResource>>,
    mut focus: ResMut<InputFocus>,
) {
    for event in events.read() {
        let Ok((_, mut input)) = inputs.get_mut(event.entity) else {
            continue;
        };
        match event.action {
            TextInputAction::Submit => {
                let value = input.value().to_string();
                let message = value.trim();
                if !message.is_empty()
                    && let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Chat {
                        content: ChatContent::Text(message.to_owned()),
                    })
                {
                    input.clear();
                }
            }
            TextInputAction::Cancel => {
                focus.clear();
                chat.quick_voice_open = false;
            }
        }
    }
    for (_, input) in &inputs {
        if input.value() != chat.input.as_str() {
            chat.input = input.value().to_string();
        }
    }
}
