//! 德州扑克的客户端筹码账本、找零算法和牌桌筹码动画。
//!
//! 规则核心仍以整数结算；这里把整数映射为有独立身份的实体筹码。每枚筹码在
//! 玩家余额区、玩家下注区和中央底池之间转移，UI 重建时也能从资源恢复位置。

use super::super::TexasAudioCue;
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, SeatId};
use std::collections::HashMap;

pub(super) const DENOMINATIONS: [u16; 5] = [100, 25, 10, 5, 1];
pub(super) const CHIP_SIZE: f32 = 34.0;
pub(super) const CHIP_MOVE_DURATION: f32 = 0.42;
pub(super) const PLAYER_CHIP_ZONE_WIDTH: f32 = 210.0;
pub(super) const PLAYER_CHIP_ZONE_HEIGHT: f32 = 130.0;
pub(super) const TEXAS_CHIP_ZONE_FILTER: Color = Color::srgba(0.005, 0.018, 0.014, 0.26);

#[derive(Component)]
pub(super) struct TexasChipZonePanel;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum ChipZone {
    Stack(PlayerId),
    Bet(PlayerId),
    Pot(usize),
    Retired,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ChipMotion {
    pub(super) start: Vec2,
    pub(super) target: Vec2,
    pub(super) start_rotation: f32,
    pub(super) target_rotation: f32,
    pub(super) elapsed: f32,
    pub(super) delay: f32,
    pub(super) duration: f32,
}

#[derive(Clone, Debug)]
pub(super) struct TableChip {
    pub(super) id: u64,
    pub(super) denomination: u16,
    pub(super) zone: ChipZone,
    pub(super) position: Vec2,
    pub(super) rotation: f32,
    pub(super) motion: Option<ChipMotion>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ActionLabel {
    pub(super) text: &'static str,
    pub(super) font_size: f32,
    pub(super) color: Color,
    pub(super) folded: bool,
    pub kind: ActionFeedbackKind,
    pub elapsed: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActionFeedbackKind {
    Blind,
    Fold,
    Check,
    Call,
    Raise,
    AllIn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct VisualPot {
    pub(super) amount: u32,
    pub(super) eligible: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PotDivisionTransition {
    pub(super) old_count: usize,
    pub(super) new_count: usize,
    pub(super) elapsed: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ChipZoneLayout {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl ChipZoneLayout {
    pub(super) fn center(self) -> Vec2 {
        Vec2::new(self.left + self.width * 0.5, self.top + self.height * 0.5)
    }
}

#[derive(Resource, Default)]
pub(crate) struct TexasChipTableState {
    pub(super) match_id: Option<MatchId>,
    pub(super) hand_number: u32,
    pub(super) you: Option<PlayerId>,
    pub(super) seats: HashMap<PlayerId, SeatId>,
    pub(super) chips: Vec<TableChip>,
    pub actions: HashMap<PlayerId, ActionLabel>,
    pub audio_cues: Vec<TexasAudioCue>,
    pub(super) current_player: Option<PlayerId>,
    pub(super) pots: Vec<VisualPot>,
    pub(super) division: Option<PotDivisionTransition>,
    pub(super) next_id: u64,
    pub(super) event_serial: u64,
}

#[derive(Component)]
pub(crate) struct TexasChipSprite(pub(super) u64);
