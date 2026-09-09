//! 双升手牌、计分、甩牌反馈与结算演出的状态类型。

use crate::app::presentation::{CardAnimationState, Observed};
use bevy::prelude::*;
use leocard_client::ShengjiScoreCaptureEffect;
use leocard_protocol::{MatchId, ShengjiSnapshot, ShengjiThrowFailureStage};
use leocard_shengji::ShengjiCard;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(crate) struct ShengjiUiState {
    pub selected: HashSet<ShengjiCard>,
    pub card_animations: HashMap<ShengjiCard, CardAnimationState>,
    pub observed_hand: Observed<(MatchId, u32), Vec<ShengjiCard>>,
    pub buried_open: bool,
}

impl ShengjiUiState {
    pub(crate) fn reconcile(&mut self, game: &ShengjiSnapshot) {
        self.selected.retain(|card| game.your_hand.contains(card));
    }

    pub(crate) fn clear(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.observed_hand.clear();
        self.buried_open = false;
    }
}

#[derive(Resource, Default)]
pub(crate) struct ShengjiScoreCaptureEffectState {
    pub seen_serial: u64,
    pub active: Option<ActiveShengjiScoreCapture>,
}

#[derive(Resource, Default)]
pub(crate) struct ShengjiSettlementAnimation {
    pub settlement_id: Option<MatchId>,
    pub elapsed: f32,
    pub absorption_spawned: bool,
    pub outcome_sound_played: bool,
}

#[derive(Clone)]
pub(crate) struct ActiveShengjiScoreCapture {
    pub capture: ShengjiScoreCaptureEffect,
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct ShengjiScoreTrayAnchor;

#[derive(Component)]
pub(super) struct ShengjiCollectingScoreText;

#[derive(Component)]
pub(crate) struct ShengjiKittyRevealCard {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct ShengjiKittyScoreAnchor;

#[derive(Component)]
pub(crate) struct ShengjiKittyScoreText {
    pub base: u32,
    pub awarded: u32,
}

#[derive(Component)]
pub(crate) struct ShengjiKittyMultiplier;

#[derive(Component)]
pub(crate) struct ShengjiSettlementTotalAnchor;

#[derive(Component)]
pub(crate) struct ShengjiSettlementTotalText {
    pub target: u32,
}

#[derive(Component)]
pub(crate) struct ShengjiSettlementModal;

#[derive(Component)]
pub(crate) struct ShengjiSettlementRow {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct ShengjiSettlementActions {
    pub delay: f32,
}

#[derive(Component)]
pub(super) struct ShengjiSettlementOutcomeText;

#[derive(Component)]
pub(super) struct ShengjiFailedThrowCard {
    pub index: usize,
    pub count: usize,
    pub stage: ShengjiThrowFailureStage,
    pub direction: Vec2,
    pub elapsed: f32,
}

#[derive(Component)]
pub(super) struct ShengjiFailedThrowLabel {
    pub returning: bool,
    pub elapsed: f32,
}

#[derive(Component)]
pub(super) struct ShengjiThrowPenaltyFloat {
    pub source: Vec2,
    pub target: Vec2,
    pub elapsed: f32,
}

#[derive(Component)]
pub(super) struct ShengjiThrowPenaltyScorePulse {
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct ShengjiDealerBadge;

#[derive(Component)]
pub(crate) struct ShengjiLevelIndicator {
    pub base_color: Color,
}

#[derive(Component)]
pub(crate) struct ShengjiTimedReveal {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct ActiveShengjiScoreAbsorb {
    pub source: Vec2,
    pub target: Vec2,
    pub elapsed: f32,
    pub delay: f32,
    pub curve: f32,
}

#[derive(Component)]
pub(crate) struct ShengjiHandCardSlot {
    pub card: ShengjiCard,
    pub index: usize,
    pub hand_len: usize,
    pub is_last: bool,
    pub hover_amount: f32,
}

#[derive(Component)]
pub(crate) struct ShengjiHandCardVisual {
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
pub(crate) struct ShengjiHandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct ShengjiSettlementPanelTexture;
