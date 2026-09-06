//! 聊天抽屉、快捷语音与表情按钮动作。

use super::*;
use leocard_protocol::{ChatContent, ClientCommand};

pub fn handle_chat_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    form: &mut ConnectionForm,
    chat: &mut ChatPanelState,
    developer_hand: &mut DeveloperHandInput,
) -> bool {
    #[cfg(not(feature = "developer"))]
    let _ = &form;
    match action {
        #[cfg(feature = "developer")]
        UiAction::FocusDeveloperHand => {
            chat.focused = false;
            developer_hand.focused = true;
            form.active = InputField::PlayerName;
            form.error = None;
        }
        UiAction::ToggleChatPanel => {
            developer_hand.focused = false;
            chat.open = !chat.open;
            if !chat.open {
                chat.focused = false;
                chat.quick_voice_open = false;
                chat.emoji_open = false;
            }
        }
        UiAction::FocusChatInput => {
            developer_hand.focused = false;
            chat.open = true;
            chat.focused = true;
            chat.quick_voice_open = false;
            chat.emoji_open = false;
        }
        UiAction::ToggleQuickVoiceMenu => {
            chat.open = true;
            chat.focused = false;
            chat.quick_voice_open = !chat.quick_voice_open;
            chat.emoji_open = false;
        }
        UiAction::ToggleEmojiMenu => {
            chat.open = true;
            chat.focused = false;
            chat.emoji_open = !chat.emoji_open;
            chat.quick_voice_open = false;
        }
        UiAction::SendQuickVoice(index) => {
            if let Some(client) = client.as_deref_mut() {
                client.0.send(ClientCommand::Chat {
                    content: ChatContent::QuickVoice(*index),
                });
            }
            chat.quick_voice_open = false;
        }
        UiAction::SendEmoji(emoji) => {
            if let Some(client) = client.as_deref_mut() {
                client.0.send(ClientCommand::Chat {
                    content: ChatContent::Emoji(*emoji),
                });
            }
            chat.emoji_open = false;
        }
        _ => return false,
    }
    true
}
