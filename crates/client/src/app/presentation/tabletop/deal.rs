use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;

pub(crate) const CARD_DEAL_DURATION: f32 = 0.28;
pub(crate) const CARD_DEAL_INTERVAL: f32 = 0.07;

#[derive(Component)]
pub(crate) struct TablePieceDeal {
    elapsed: f32,
    delay: f32,
    offset: Option<Vec2>,
    source: Option<Entity>,
    duration: f32,
    fade_in: Option<f32>,
    transient: bool,
    rotation: Option<(f32, f32)>,
    finished: bool,
}

impl TablePieceDeal {
    pub(crate) fn new(delay: f32, offset: Vec2) -> Self {
        Self {
            elapsed: 0.0,
            delay,
            offset: Some(offset),
            source: None,
            duration: CARD_DEAL_DURATION,
            fade_in: None,
            transient: false,
            rotation: None,
            finished: false,
        }
    }

    pub(crate) fn from_anchor(source: Entity, delay: f32) -> Self {
        Self {
            elapsed: 0.0,
            delay,
            offset: None,
            source: Some(source),
            duration: CARD_DEAL_DURATION,
            fade_in: None,
            transient: false,
            rotation: None,
            finished: false,
        }
    }

    pub(crate) fn transient(mut self, duration: f32) -> Self {
        self.transient = true;
        self.duration = duration;
        self
    }

    pub(crate) fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub(crate) fn with_fade_in(mut self, duration: f32) -> Self {
        self.fade_in = Some(duration);
        self
    }

    pub(crate) fn with_rotation(mut self, start: f32, target: f32) -> Self {
        self.rotation = Some((start, target));
        self
    }

    pub(super) fn ready(&self) -> bool {
        self.offset.is_some()
    }

    pub(super) fn scale(&self) -> f32 {
        if self.elapsed < self.delay {
            0.0
        } else {
            0.76 + ease_out_cubic(self.progress()) * 0.24
        }
    }

    fn progress(&self) -> f32 {
        ((self.elapsed - self.delay) / self.duration).clamp(0.0, 1.0)
    }
}

/// Measure the actual laid-out destination, including UI scaling, before flight.
pub(crate) fn resolve_table_piece_origins(
    mut cards: Query<(&mut TablePieceDeal, &ComputedNode, &UiGlobalTransform)>,
    transforms: Query<&UiGlobalTransform>,
) {
    for (mut deal, node, target) in &mut cards {
        if deal.ready() || node.size().min_element() <= 0.0 {
            continue;
        }
        let Some(source) = deal.source.and_then(|source| transforms.get(source).ok()) else {
            continue;
        };
        let delta = source.affine().translation - target.affine().translation;
        deal.offset = Some(delta * node.inverse_scale_factor());
    }
}

pub(crate) fn animate_table_piece_deals(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(
        Entity,
        &mut TablePieceDeal,
        &mut UiTransform,
        Option<&mut ImageNode>,
    )>,
) {
    for (entity, mut deal, mut transform, mut image) in &mut cards {
        if deal.finished {
            continue;
        }
        let Some(offset) = deal.offset else {
            if let Some(image) = image.as_deref_mut() {
                image.color = Color::WHITE.with_alpha(0.0);
            }
            transform.scale = Vec2::ZERO;
            continue;
        };
        deal.elapsed += time.delta_secs();
        let progress = deal.progress();
        let movement = ease_out_cubic(progress);
        transform.translation = Val2::px(offset.x * (1.0 - movement), offset.y * (1.0 - movement));
        transform.scale = Vec2::splat(deal.scale());
        if let Some((start, target)) = deal.rotation {
            transform.rotation = Rot2::radians(start + (target - start) * movement);
        }
        if let Some(image) = image.as_deref_mut() {
            let alpha = ((deal.elapsed - deal.delay) / deal.fade_in.unwrap_or(deal.duration))
                .clamp(0.0, 1.0);
            image.color = Color::WHITE.with_alpha(alpha);
        }
        if progress >= 1.0 {
            deal.finished = true;
            if deal.transient {
                commands.entity(entity).despawn();
            }
        }
    }
}
