//! 七鬼五二三手牌动画与牌型演出状态。

use super::QiGui523Assets;
use crate::app::presentation::{CardAnimationState, Observed};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{ScoreCaptureEffectState, SeatSide};
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, PublicPlay, QiGui523Snapshot};
use leocard_qigui523::{QiGui523Bot, QiGuiCard};
use std::collections::{HashMap, HashSet};

pub(super) struct SeatVisuals<'a> {
    pub(super) game: &'a QiGui523Snapshot,
    pub(super) client: &'a ClientResource,
    pub(super) ui: &'a UiAssets,
    pub(super) assets: &'a QiGui523Assets,
    pub(super) avatars: &'a AvatarImages,
    pub(super) interaction_menu_open: Option<PlayerId>,
    pub(super) play_effect: Option<&'a ActivePlayEffect>,
    pub(super) last_play: Option<&'a (PlayerId, PublicPlay)>,
    pub(super) score_capture: &'a ScoreCaptureEffectState,
    pub(super) start_transition_active: bool,
}

#[derive(Clone, Copy)]
pub(super) enum ScoreCardsPopupPlacement {
    Opponent(SeatSide),
    Own,
}

#[derive(Resource, Default)]
pub(crate) struct QiGui523UiState {
    pub selected: HashSet<QiGuiCard>,
    pub card_animations: HashMap<QiGuiCard, CardAnimationState>,
    pub observed_hand: Observed<MatchId, Vec<QiGuiCard>>,
    pub greedy_hint: QiGui523Bot,
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
        self.greedy_hint.reset();
    }
}

#[derive(Resource, Default)]
pub(crate) struct PlayEffectState {
    pub seen_serial: u64,
    pub active: Option<ActivePlayEffect>,
}

#[derive(Clone)]
pub(crate) struct ActivePlayEffect {
    pub player: PlayerId,
    pub play: PublicPlay,
    pub elapsed: f32,
    pub bomb_sound_played: bool,
}

#[derive(Component)]
pub(crate) struct PlayEffectRoot;

#[derive(Component)]
pub(crate) struct SequenceEffectCard {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct SequenceGuideSegment {
    pub index: usize,
    pub count: usize,
}

#[derive(Component)]
pub(crate) struct SequenceEffectLabel;

#[derive(Component, Clone, Copy)]
pub(crate) struct SequenceEffectLabelPart {
    pub outline: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SequenceEffectMotif {
    Wind,
    Flower,
    Airplane,
}

#[derive(Component)]
pub(crate) struct SequenceWindStreak {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct SequenceFlowerPart {
    pub index: usize,
    pub petal: bool,
}

#[derive(Component)]
pub(crate) struct SequenceAirplane;

#[derive(Component)]
pub(crate) struct SequenceAirplaneTrail {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct BombEffectBody {
    pub source: Vec2,
}

#[derive(Component)]
pub(crate) struct BombFuseSpark;

#[derive(Component)]
pub(crate) struct BombExplosionFlash;

#[derive(Component)]
pub(crate) struct BombExplosionRing;

#[derive(Component)]
pub(crate) struct BombExplosionParticle {
    pub direction: Vec2,
    pub distance: f32,
}

#[derive(Component)]
pub(crate) struct HeavenBombBackdrop;

#[derive(Component)]
pub(crate) struct HeavenBombFlash;

#[derive(Component)]
pub(crate) struct HeavenBombRay {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct HeavenBombShockRing {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct HeavenBombParticle {
    pub direction: Vec2,
    pub distance: f32,
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct HeavenBombTitle;

#[derive(Component)]
pub(crate) struct HeavenBombTitleText;

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
pub(crate) struct HandCardSlot {
    pub card: QiGuiCard,
    pub index: usize,
    pub is_last: bool,
    pub hover_amount: f32,
}
