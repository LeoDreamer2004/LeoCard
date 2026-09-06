//! 聊天界面、消息同步与气泡演出。

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_protocol::{ChatContent, ChatEmoji, ClientCommand, PlayerId, QUICK_VOICE_COUNT};
use std::collections::{HashMap, VecDeque};

use super::*;

pub const CHAT_PANEL_WIDTH: f32 = 350.0;
/// 将面板本体移出右侧，同时保留其左侧的 32px 折叠箭头。
pub const CHAT_PANEL_HIDDEN_OFFSET: f32 = CHAT_PANEL_WIDTH + 2.0;
pub const QUICK_VOICES: [&str; QUICK_VOICE_COUNT as usize] = [
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

pub mod actions;
mod presentation;
mod state;
mod view;

pub use presentation::*;
pub use state::*;
pub use view::*;
