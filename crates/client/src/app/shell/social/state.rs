//! 玩家互动菜单、冷却与飞行道具的状态类型。

use bevy::prelude::*;
use leocard_client::ScoreCaptureEffect;
use leocard_protocol::{PlayerId, PlayerInteractionKind};
use std::collections::HashMap;

pub(crate) const INTERACTION_COOLDOWN_MASK_FRAMES: usize = 48;

#[derive(Default)]
pub(crate) struct SocialUiState {
    pub interaction_menu_open: Option<PlayerId>,
}

#[derive(Resource, Default)]
pub(crate) struct ScoreCaptureEffectState {
    pub seen_serial: u64,
    pub active: Option<ActiveScoreCapture>,
}

#[derive(Clone)]
pub(crate) struct ActiveScoreCapture {
    pub capture: ScoreCaptureEffect,
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct PlayerInteractionLayer;

#[derive(Component)]
pub(crate) struct FinishedHandScoreSource(pub PlayerId);

#[derive(Component)]
pub(crate) struct ActiveScoreCaptureCard {
    pub source: Vec2,
    pub target: Vec2,
    pub elapsed: f32,
    pub delay: f32,
    pub curve: f32,
}

#[derive(Component)]
pub(crate) struct ActiveScoreVortex {
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct ActiveScoreGainText {
    pub elapsed: f32,
    pub text: Entity,
}

/// 玩家框相对于牌桌的方位；多个游戏与公共弹窗布局共同使用。
#[derive(Clone, Copy)]
pub(crate) enum SeatSide {
    Left,
    Top,
    Right,
}

#[derive(Clone, Copy, Component)]
pub(crate) enum PlayerGameScoreText {
    Opponent { player: PlayerId, side: SeatSide },
    Own(PlayerId),
}

impl PlayerGameScoreText {
    pub(crate) fn player(self) -> PlayerId {
        match self {
            Self::Opponent { player, .. } | Self::Own(player) => player,
        }
    }

    pub(crate) fn label(self, score: u32) -> String {
        match self {
            Self::Opponent { .. } | Self::Own(_) => score.to_string(),
        }
    }

    pub(crate) fn gain_left(self, center_x: f32, width: f32) -> f32 {
        match self {
            Self::Opponent {
                side: SeatSide::Right,
                ..
            } => center_x - width * 0.5 - 84.0,
            Self::Opponent { .. } | Self::Own(_) => center_x + width * 0.5 + 8.0,
        }
    }
}

#[derive(Component)]
pub(crate) struct AutoPlayRobotIndicator;

#[derive(Clone, Copy, Component)]
pub(crate) struct AutoPlayAntennaLight {
    pub player: PlayerId,
    pub part: AutoPlayAntennaLightPart,
}

#[derive(Clone, Copy)]
pub(crate) enum AutoPlayAntennaLightPart {
    Glow,
    Ray,
}

#[derive(Resource, Default)]
pub(crate) struct PlayerInteractionCooldown {
    pub timers: HashMap<PlayerInteractionKind, InteractionCooldownTimer>,
}

#[derive(Clone, Copy)]
pub(crate) struct InteractionCooldownTimer {
    pub remaining: f32,
    pub duration: f32,
}

impl PlayerInteractionCooldown {
    pub(crate) fn is_active(&self, kind: PlayerInteractionKind) -> bool {
        self.timers
            .get(&kind)
            .is_some_and(|timer| timer.remaining > 0.0)
    }

    pub(crate) fn start(&mut self, kind: PlayerInteractionKind, duration: f32) {
        self.timers.insert(
            kind,
            InteractionCooldownTimer {
                remaining: duration,
                duration,
            },
        );
    }

    pub(crate) fn tick(&mut self, delta: f32) {
        for timer in self.timers.values_mut() {
            timer.remaining = (timer.remaining - delta).max(0.0);
        }
        self.timers.retain(|_, timer| timer.remaining > 0.0);
    }

    pub(crate) fn fraction(&self, kind: PlayerInteractionKind) -> f32 {
        self.timers.get(&kind).map_or(0.0, |timer| {
            (timer.remaining / timer.duration).clamp(0.0, 1.0)
        })
    }
}

#[derive(Component)]
pub(crate) struct OpponentBadge {
    pub player: PlayerId,
    pub score_popup: Option<Entity>,
    pub interaction_menu: Entity,
}

#[derive(Component)]
pub(crate) struct PlayerAvatarAnchor(pub PlayerId);

#[derive(Component)]
pub(crate) struct InteractionMenuPanel(pub PlayerId);

#[derive(Component)]
pub(crate) struct InteractionCooldownMask {
    pub player: PlayerId,
    pub kind: PlayerInteractionKind,
}

#[derive(Component)]
pub(crate) struct ActivePlayerInteraction {
    pub source: Vec2,
    pub target: Vec2,
    pub kind: PlayerInteractionKind,
    pub sound_variant: u8,
    pub play_sound: bool,
    pub elapsed: f32,
    pub appear_duration: f32,
    pub travel_duration: f32,
    pub impact_duration: f32,
    pub launched: bool,
    pub impacted: bool,
}
