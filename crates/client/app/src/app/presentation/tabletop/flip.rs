use super::TablePieceDeal;
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayingCardFlip {
    elapsed: f32,
    delay: f32,
    duration: f32,
    face: Handle<Image>,
    back: Handle<Image>,
    face_visible: bool,
    finished: bool,
}

pub(crate) struct PlayingCardFlipFrame {
    pub texture: Handle<Image>,
    pub scale: Vec2,
}

impl PlayingCardFlip {
    pub(crate) const DEFAULT_DURATION: f32 = 0.34;

    pub(crate) fn new(face: Handle<Image>, back: Handle<Image>, delay: f32) -> Self {
        Self {
            elapsed: 0.0,
            delay,
            duration: Self::DEFAULT_DURATION,
            face,
            back,
            face_visible: false,
            finished: false,
        }
    }

    pub(crate) fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub(crate) fn has_revealed(&self) -> bool {
        self.face_visible
    }

    pub(crate) fn advance(
        &mut self,
        delta: f32,
        deal: Option<&TablePieceDeal>,
    ) -> Option<PlayingCardFlipFrame> {
        if self.finished || deal.is_some_and(|deal| !deal.ready()) {
            return None;
        }
        self.elapsed += delta;
        let progress = ((self.elapsed - self.delay) / self.duration).clamp(0.0, 1.0);
        self.face_visible = progress >= 0.5;
        self.finished = progress >= 1.0;
        let mut frame = self.frame_at(self.elapsed);
        frame.scale *= deal.map_or(1.0, TablePieceDeal::scale);
        Some(frame)
    }

    pub(crate) fn frame_at(&self, elapsed: f32) -> PlayingCardFlipFrame {
        let progress = ((elapsed - self.delay) / self.duration).clamp(0.0, 1.0);
        let texture = if progress >= 0.5 {
            &self.face
        } else {
            &self.back
        }
        .clone();
        let horizontal = if progress < 0.5 {
            1.0 - ease_out_cubic(progress * 2.0)
        } else {
            ease_out_cubic((progress - 0.5) * 2.0)
        };
        PlayingCardFlipFrame {
            texture,
            scale: Vec2::new(horizontal.max(0.025), 1.0 + (1.0 - horizontal) * 0.04),
        }
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
        if let Some(frame) = flip.advance(time.delta_secs(), deal) {
            image.image = frame.texture;
            transform.scale = frame.scale;
        }
    }
}
