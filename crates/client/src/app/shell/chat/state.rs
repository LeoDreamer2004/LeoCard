//! 聊天抽屉、历史记录与气泡实体的状态类型。

use super::*;

#[derive(Clone, Debug)]
pub struct ChatHistoryEntry {
    pub player_name: String,
    pub message: String,
}

#[derive(Resource)]
pub struct ChatPanelState {
    pub open: bool,
    pub slide: f32,
    pub focused: bool,
    pub quick_voice_open: bool,
    pub emoji_open: bool,
    pub quick_voice_scroll_y: f32,
    pub emoji_scroll_y: f32,
    pub input: String,
    pub history: VecDeque<ChatHistoryEntry>,
}

impl Default for ChatPanelState {
    fn default() -> Self {
        Self {
            open: false,
            slide: 1.0,
            focused: false,
            quick_voice_open: false,
            emoji_open: false,
            quick_voice_scroll_y: 0.0,
            emoji_scroll_y: 0.0,
            input: String::new(),
            history: VecDeque::new(),
        }
    }
}

#[derive(Component)]
pub struct ChatPanel;

#[derive(Component)]
pub struct ChatToggleIcon;

#[derive(Component)]
pub struct ChatHistoryText;

#[derive(Component)]
pub struct ChatInputText;

#[derive(Component)]
pub struct EmojiMenu;

#[derive(Component)]
pub struct EmojiScroll;

#[derive(Component)]
pub struct QuickVoiceMenu;

#[derive(Component)]
pub struct QuickVoiceScroll;

#[derive(Component)]
pub struct ActiveChatBubble {
    pub player: PlayerId,
    pub text: Option<Entity>,
    pub emoji_image: Option<Entity>,
    pub emoji: bool,
    pub width: f32,
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct ChatBubbleText;
