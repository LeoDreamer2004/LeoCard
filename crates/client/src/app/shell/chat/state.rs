//! 聊天抽屉、历史记录与气泡实体的状态类型。

use bevy::prelude::*;
use leocard_protocol::{PlayerId, QUICK_VOICE_COUNT};
use std::collections::VecDeque;

pub(super) const CHAT_PANEL_WIDTH: f32 = 350.0;
/// 将面板本体移出右侧，同时保留其左侧的 32px 折叠箭头。
pub(super) const CHAT_PANEL_HIDDEN_OFFSET: f32 = CHAT_PANEL_WIDTH + 2.0;

pub(crate) const QUICK_VOICES: [&str; QUICK_VOICE_COUNT as usize] = [
    "我从未见过如此厚颜无耻之人！",
    "这波不亏",
    "请收下我的膝盖",
    "你咋不上天呢",
    "放开我的队友，冲我来",
    "你随便杀，闪不了算我输",
    "见证奇迹的时刻到了",
    "能不能快一点啊，兵贵神速啊",
    "主公，别开枪，自己人",
    "小内再不跳，后面还怎么玩儿啊",
    "你们忍心，就这么让我酱油了？",
    "我，我惹你们了吗",
    "姑娘，你真是条汉子",
    "三十六计，走为上，容我去去便回",
    "人心散了，队伍不好带啊",
    "昏君，昏君啊！",
    "风吹鸡蛋壳，牌去人安乐",
    "小内啊，您老悠着点儿",
    "不好意思，刚才卡了",
    "你可以打得再烂一点吗",
    "哥们，给力点儿行嘛",
    "哥哥，交个朋友吧",
    "妹子，交个朋友吧",
];

#[derive(Clone, Debug)]
pub(crate) struct ChatHistoryEntry {
    pub player_name: String,
    pub message: String,
}

#[derive(Resource)]
pub(crate) struct ChatPanelState {
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
pub(crate) struct ChatPanel;

#[derive(Component)]
pub(crate) struct ChatToggleIcon;

#[derive(Component)]
pub(crate) struct ChatHistoryText;

#[derive(Component)]
pub(crate) struct ChatInputText;

#[derive(Component)]
pub(crate) struct EmojiMenu;

#[derive(Component)]
pub(crate) struct EmojiScroll;

#[derive(Component)]
pub(crate) struct QuickVoiceMenu;

#[derive(Component)]
pub(crate) struct QuickVoiceScroll;

#[derive(Component)]
pub(crate) struct ActiveChatBubble {
    pub player: PlayerId,
    pub text: Option<Entity>,
    pub emoji_image: Option<Entity>,
    pub emoji: bool,
    pub width: f32,
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Component)]
pub(crate) struct ChatBubbleText;
