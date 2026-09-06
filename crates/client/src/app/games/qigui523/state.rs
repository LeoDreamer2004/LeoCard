//! 七鬼五二三手牌动画与牌型演出状态。

use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct QiGui523UiState {
    pub selected: HashSet<QiGuiCard>,
    pub card_animations: HashMap<QiGuiCard, CardAnimationState>,
    pub observed_hand: Vec<QiGuiCard>,
    pub greedy_hint: QiGui523Bot,
}

#[derive(Resource, Default)]
pub struct PlayEffectState {
    pub seen_serial: u64,
    pub active: Option<ActivePlayEffect>,
}

#[derive(Clone)]
pub struct ActivePlayEffect {
    pub player: PlayerId,
    pub play: PublicPlay,
    pub elapsed: f32,
    pub bomb_sound_played: bool,
}

#[derive(Component)]
pub struct PlayEffectRoot;

#[derive(Component)]
pub struct SequenceEffectCard {
    pub index: usize,
}

#[derive(Component)]
pub struct SequenceGuideSegment {
    pub index: usize,
    pub count: usize,
}

#[derive(Component)]
pub struct SequenceEffectLabel;

#[derive(Component, Clone, Copy)]
pub struct SequenceEffectLabelPart {
    pub outline: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceEffectMotif {
    Wind,
    Flower,
    Airplane,
}

#[derive(Component)]
pub struct SequenceWindStreak {
    pub index: usize,
}

#[derive(Component)]
pub struct SequenceFlowerPart {
    pub index: usize,
    pub petal: bool,
}

#[derive(Component)]
pub struct SequenceAirplane;

#[derive(Component)]
pub struct SequenceAirplaneTrail {
    pub index: usize,
}

#[derive(Component)]
pub struct BombEffectBody {
    pub source: Vec2,
}

#[derive(Component)]
pub struct BombFuseSpark;

#[derive(Component)]
pub struct BombExplosionFlash;

#[derive(Component)]
pub struct BombExplosionRing;

#[derive(Component)]
pub struct BombExplosionParticle {
    pub direction: Vec2,
    pub distance: f32,
}

#[derive(Component)]
pub struct HeavenBombBackdrop;

#[derive(Component)]
pub struct HeavenBombFlash;

#[derive(Component)]
pub struct HeavenBombRay {
    pub index: usize,
}

#[derive(Component)]
pub struct HeavenBombShockRing {
    pub delay: f32,
}

#[derive(Component)]
pub struct HeavenBombParticle {
    pub direction: Vec2,
    pub distance: f32,
    pub delay: f32,
}

#[derive(Component)]
pub struct HeavenBombTitle;

#[derive(Component)]
pub struct HeavenBombTitleText;

#[derive(Component)]
pub struct HandCardVisual {
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
pub struct HandCardSlot {
    pub card: QiGuiCard,
    pub index: usize,
    pub is_last: bool,
    pub hover_amount: f32,
}

#[derive(Clone, Copy)]
pub struct HandCardPose {
    pub translation: Val2,
    pub rotation: Rot2,
}
