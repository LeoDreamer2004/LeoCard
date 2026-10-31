//! 共用出牌特效的类型、播放状态和视觉组件。

use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::MatchId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayEffectKind {
    Straight,
    ConsecutivePairs,
    Airplane,
    Bomb,
    HeavenBomb,
}

impl PlayEffectKind {
    pub(crate) fn is_sequence(self) -> bool {
        self.sequence_style().is_some()
    }

    pub(super) fn sequence_style(self) -> Option<(&'static str, Color, SequenceEffectMotif)> {
        match self {
            Self::Straight => Some((
                "顺子",
                Color::srgb(0.25, 0.78, 0.96),
                SequenceEffectMotif::Wind,
            )),
            Self::ConsecutivePairs => Some((
                "连对",
                Color::srgb(0.76, 0.46, 0.98),
                SequenceEffectMotif::Flower,
            )),
            Self::Airplane => Some((
                "飞机",
                Color::srgb(0.98, 0.36, 0.50),
                SequenceEffectMotif::Airplane,
            )),
            Self::Bomb | Self::HeavenBomb => None,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct PlayEffectState {
    pub active: Option<ActivePlayEffect>,
}

impl PlayEffectState {
    pub(crate) fn start(
        &mut self,
        match_id: MatchId,
        player: PlayerId,
        kind: PlayEffectKind,
        card_count: usize,
    ) {
        self.active = Some(ActivePlayEffect {
            match_id,
            player,
            kind,
            card_count,
            elapsed: 0.0,
            bomb_sound_played: false,
        });
    }
}

#[derive(Clone)]
pub(crate) struct ActivePlayEffect {
    pub match_id: MatchId,
    pub player: PlayerId,
    pub kind: PlayEffectKind,
    pub card_count: usize,
    pub elapsed: f32,
    pub(super) bomb_sound_played: bool,
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
