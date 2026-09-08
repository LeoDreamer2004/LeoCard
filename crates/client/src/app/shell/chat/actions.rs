//! 聊天抽屉、快捷语音与表情按钮动作。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{ChatContent, ClientCommand};

#[derive(SystemParam)]
pub struct ChatActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    form: ResMut<'w, ConnectionForm>,
    chat: ResMut<'w, ChatPanelState>,
    developer_hand: ResMut<'w, DeveloperHandInput>,
}

pub fn dispatch_chat_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ChatActionContext,
) {
    dispatch_domain_actions::<ChatUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ChatActionContext<'_>> for ChatUiAction {
    fn handle(&self, context: &mut ChatActionContext<'_>) {
        let client = &mut context.client;
        let form = &mut *context.form;
        let chat = &mut *context.chat;
        let developer_hand = &mut *context.developer_hand;
        #[cfg(not(feature = "developer"))]
        let _ = &form;
        match self {
            #[cfg(feature = "developer")]
            ChatUiAction::FocusDeveloperHand => {
                chat.focused = false;
                developer_hand.focused = true;
                form.active = InputField::PlayerName;
                form.error = None;
            }
            ChatUiAction::TogglePanel => {
                developer_hand.focused = false;
                chat.open = !chat.open;
                if !chat.open {
                    chat.focused = false;
                    chat.quick_voice_open = false;
                    chat.emoji_open = false;
                }
            }
            ChatUiAction::FocusInput => {
                developer_hand.focused = false;
                chat.open = true;
                chat.focused = true;
                chat.quick_voice_open = false;
                chat.emoji_open = false;
            }
            ChatUiAction::ToggleQuickVoiceMenu => {
                chat.open = true;
                chat.focused = false;
                chat.quick_voice_open = !chat.quick_voice_open;
                chat.emoji_open = false;
            }
            ChatUiAction::ToggleEmojiMenu => {
                chat.open = true;
                chat.focused = false;
                chat.emoji_open = !chat.emoji_open;
                chat.quick_voice_open = false;
            }
            ChatUiAction::SendQuickVoice(index) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Chat {
                        content: ChatContent::QuickVoice(*index),
                    });
                }
                chat.quick_voice_open = false;
            }
            ChatUiAction::SendEmoji(emoji) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Chat {
                        content: ChatContent::Emoji(*emoji),
                    });
                }
                chat.emoji_open = false;
            }
        }
    }
}
