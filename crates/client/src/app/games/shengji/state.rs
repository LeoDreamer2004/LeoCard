//! 双升手牌、计分、甩牌反馈与结算演出的状态类型。

use super::*;
use leocard_client::ShengjiScoreCaptureEffect;
use leocard_protocol::{MatchId, ShengjiThrowFailureStage};
use leocard_shengji::ShengjiCard;
use std::collections::HashMap;

#[derive(Default)]
pub struct ShengjiUiState {
    pub selected: HashSet<ShengjiCard>,
    pub card_animations: HashMap<ShengjiCard, CardAnimationState>,
    pub observed_hand: Vec<ShengjiCard>,
    pub observed_match: Option<MatchId>,
    pub observed_hand_number: u32,
    pub buried_open: bool,
}

#[derive(Resource, Default)]
pub struct ShengjiScoreCaptureEffectState {
    pub seen_serial: u64,
    pub active: Option<ActiveShengjiScoreCapture>,
}

#[derive(Resource, Default)]
pub struct ShengjiSettlementAnimation {
    pub settlement_id: Option<MatchId>,
    pub elapsed: f32,
    pub absorption_spawned: bool,
    pub outcome_sound_played: bool,
}

#[derive(Clone)]
pub struct ActiveShengjiScoreCapture {
    pub capture: ShengjiScoreCaptureEffect,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct ShengjiScoreTrayAnchor;

#[derive(Component)]
pub struct ShengjiCollectingScoreText;

#[derive(Component)]
pub struct ShengjiKittyRevealCard {
    pub index: usize,
}

#[derive(Component)]
pub struct ShengjiKittyScoreAnchor;

#[derive(Component)]
pub struct ShengjiKittyScoreText {
    pub base: u32,
    pub awarded: u32,
}

#[derive(Component)]
pub struct ShengjiKittyMultiplier;

#[derive(Component)]
pub struct ShengjiSettlementTotalAnchor;

#[derive(Component)]
pub struct ShengjiSettlementTotalText {
    pub target: u32,
}

#[derive(Component)]
pub struct ShengjiSettlementModal;

#[derive(Component)]
pub struct ShengjiSettlementRow {
    pub delay: f32,
}

#[derive(Component)]
pub struct ShengjiSettlementActions {
    pub delay: f32,
}

#[derive(Component)]
pub struct ShengjiSettlementOutcomeText;

#[derive(Component)]
pub struct ShengjiFailedThrowCard {
    pub index: usize,
    pub count: usize,
    pub stage: ShengjiThrowFailureStage,
    pub direction: Vec2,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct ShengjiFailedThrowLabel {
    pub returning: bool,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct ShengjiThrowPenaltyFloat {
    pub source: Vec2,
    pub target: Vec2,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct ShengjiThrowPenaltyScorePulse {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct ShengjiDealerBadge;

#[derive(Component)]
pub struct ShengjiLevelIndicator {
    pub base_color: Color,
}

#[derive(Component)]
pub struct ShengjiTimedReveal {
    pub delay: f32,
}

#[derive(Component)]
pub struct ActiveShengjiScoreAbsorb {
    pub source: Vec2,
    pub target: Vec2,
    pub elapsed: f32,
    pub delay: f32,
    pub curve: f32,
}

#[derive(Component)]
pub struct ShengjiHandCardSlot {
    pub card: ShengjiCard,
    pub index: usize,
    pub hand_len: usize,
    pub is_last: bool,
    pub hover_amount: f32,
}

#[derive(Component)]
pub struct ShengjiHandCardVisual {
    pub button: Entity,
    pub card: ShengjiCard,
    pub index: usize,
    pub selected: bool,
    pub hover_amount: f32,
    pub selected_amount: f32,
    pub deal_elapsed: f32,
    pub dealing: bool,
    pub hand_len: usize,
}

#[derive(Component)]
pub struct ShengjiHandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub struct ShengjiSettlementPanelTexture;
