//! 德州扑克的客户端筹码账本、找零算法和牌桌筹码动画。
//!
//! 规则核心仍以整数结算；这里把整数映射为有独立身份的实体筹码。每枚筹码在
//! 玩家余额区、玩家下注区和中央底池之间转移，UI 重建时也能从资源恢复位置。

use super::*;

mod geometry;
mod ledger;
mod systems;
mod view;

pub use geometry::*;
pub use systems::*;
pub use view::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
use leocard_protocol::PlayerGameProfiles;

const DENOMINATIONS: [u16; 5] = [100, 25, 10, 5, 1];
const CHIP_SIZE: f32 = 30.0;
const CHIP_MOVE_DURATION: f32 = 0.42;
const PLAYER_CHIP_ZONE_WIDTH: f32 = 160.0;
const PLAYER_CHIP_ZONE_HEIGHT: f32 = 82.0;
const TEXAS_CHIP_ZONE_FILTER: Color = Color::srgba(0.005, 0.018, 0.014, 0.26);

#[derive(Component)]
pub struct TexasChipZonePanel;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ChipZone {
    Stack(PlayerId),
    Bet(PlayerId),
    Pot(usize),
    Retired,
}

#[derive(Clone, Copy, Debug)]
struct ChipMotion {
    start: Vec2,
    target: Vec2,
    start_rotation: f32,
    target_rotation: f32,
    elapsed: f32,
    delay: f32,
    duration: f32,
}

#[derive(Clone, Debug)]
struct TableChip {
    id: u64,
    denomination: u16,
    zone: ChipZone,
    position: Vec2,
    rotation: f32,
    motion: Option<ChipMotion>,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionLabel {
    text: &'static str,
    font_size: f32,
    color: Color,
    folded: bool,
    pub kind: ActionFeedbackKind,
    pub elapsed: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionFeedbackKind {
    Blind,
    Fold,
    Check,
    Call,
    Raise,
    AllIn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisualPot {
    amount: u32,
    eligible: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug)]
struct PotDivisionTransition {
    old_count: usize,
    new_count: usize,
    elapsed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ChipZoneLayout {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl ChipZoneLayout {
    fn center(self) -> Vec2 {
        Vec2::new(self.left + self.width * 0.5, self.top + self.height * 0.5)
    }
}

#[derive(Resource, Default)]
pub struct TexasChipTableState {
    match_id: Option<MatchId>,
    hand_number: u32,
    you: Option<PlayerId>,
    seats: HashMap<PlayerId, SeatId>,
    chips: Vec<TableChip>,
    pub actions: HashMap<PlayerId, ActionLabel>,
    pub audio_cues: Vec<TexasAudioCue>,
    current_player: Option<PlayerId>,
    pots: Vec<VisualPot>,
    division: Option<PotDivisionTransition>,
    next_id: u64,
    event_serial: u64,
}

#[derive(Component)]
pub struct TexasChipSprite(u64);
