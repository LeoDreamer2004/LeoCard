use super::TablePieceDeal;
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;

pub(crate) const CARD_FLIP_DURATION: f32 = 0.34;

#[derive(Component)]
pub(crate) struct PlayingCardFlip {
    elapsed: f32,
    delay: f32,
    face: Handle<Image>,
    back: Handle<Image>,
    face_visible: bool,
    finished: bool,
}

impl PlayingCardFlip {
    pub(crate) fn new(face: Handle<Image>, back: Handle<Image>, delay: f32) -> Self {
        Self {
            elapsed: 0.0,
            delay,
            face,
            back,
            face_visible: false,
            finished: false,
        }
    }

    pub(crate) fn has_revealed(&self) -> bool {
        self.face_visible
    }
}

pub(crate) fn animate_playing_card_flips(
    time: Res<Time>,
    mut cards: Query<(
        &mut PlayingCardFlip,
        Option<&TablePieceDeal>,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (mut flip, deal, mut transform, mut image) in &mut cards {
        if flip.finished || deal.is_some_and(|deal| !deal.ready()) {
            continue;
        }
        flip.elapsed += time.delta_secs();
        let progress = ((flip.elapsed - flip.delay) / CARD_FLIP_DURATION).clamp(0.0, 1.0);
        flip.face_visible = progress >= 0.5;
        image.image = if flip.face_visible {
            flip.face.clone()
        } else {
            flip.back.clone()
        };
        let horizontal = if progress < 0.5 {
            1.0 - ease_out_cubic(progress * 2.0)
        } else {
            ease_out_cubic((progress - 0.5) * 2.0)
        };
        let scale = deal.map_or(1.0, TablePieceDeal::scale);
        transform.scale = Vec2::new(horizontal.max(0.025), 1.0 + (1.0 - horizontal) * 0.04) * scale;
        flip.finished = progress >= 1.0;
    }
}
