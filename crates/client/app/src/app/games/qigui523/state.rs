//! 七鬼五二三手牌动画与牌型演出状态。

use super::counter::QiGuiCounterUi;
use super::{QiGui523Assets, QiGuiClientState};
use crate::app::presentation::{
    ActivePlayEffect, CardAnimationState, Observed, SeatSide, SilentButton,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::ScoreCaptureEffectState;
use leocard_game_common::PlayerId;

use bevy::prelude::*;
use leocard_protocol::{MatchId, QiGui523PublicPlay, QiGui523Snapshot};
use leocard_qigui523::{QiGui523Bot, QiGuiCard};
use leocard_ui::{CustomButtonMotion, DraggableWindowOwner, DraggableWindowState};
use std::collections::{HashMap, HashSet};

pub(super) struct SeatVisuals<'a> {
    pub client_state: &'a QiGuiClientState,
    pub(super) game: &'a QiGui523Snapshot,
    pub(super) ui: &'a UiAssets,
    pub(super) assets: &'a QiGui523Assets,
    pub(super) avatars: &'a AvatarImages,
    pub(super) interaction_menu_open: Option<PlayerId>,
    pub(super) play_effect: Option<&'a ActivePlayEffect>,
    pub(super) last_play: Option<&'a (PlayerId, QiGui523PublicPlay)>,
    pub(super) score_capture: &'a ScoreCaptureEffectState,
    pub(super) start_transition_active: bool,
    pub(super) intro_only: bool,
}

#[derive(Clone, Copy)]
pub(super) enum ScoreCardsPopupPlacement {
    Opponent { side: SeatSide, above: bool },
    Own,
}

#[derive(Resource, Default)]
pub(crate) struct QiGui523UiState {
    pub selected: HashSet<QiGuiCard>,
    pub card_animations: HashMap<QiGuiCard, CardAnimationState>,
    pub observed_hand: Observed<MatchId, Vec<QiGuiCard>>,
    pub play_effect_key: Option<(MatchId, PlayerId, u64)>,
    pub greedy_hint: QiGui523Bot,
    pub counter: QiGuiCounterUi,
}

impl QiGui523UiState {
    pub(crate) fn reconcile(&mut self, game: &QiGui523Snapshot) {
        self.selected.retain(|card| game.your_hand.contains(card));
        self.card_animations
            .retain(|card, _| game.your_hand.contains(card));
    }

    pub(crate) fn clear(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.observed_hand.clear();
        self.play_effect_key = None;
        self.greedy_hint.reset();
        self.counter = QiGuiCounterUi::default();
    }
}

#[derive(Component)]
pub(super) struct HandCardVisual {
    pub button: Entity,
    pub card: QiGuiCard,
    pub index: usize,
    pub selected: bool,
    pub hover_amount: f32,
    pub selected_amount: f32,
    pub deal_elapsed: f32,
    pub dealing: bool,
    pub hand_len: usize,
}

#[derive(Component)]
#[require(CustomButtonMotion, SilentButton)]
pub(crate) struct HandCardSlot {
    pub card: QiGuiCard,
    pub index: usize,
    pub is_last: bool,
    pub hover_amount: f32,
}

impl DraggableWindowOwner for QiGui523UiState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.counter.window.drag
    }
}
