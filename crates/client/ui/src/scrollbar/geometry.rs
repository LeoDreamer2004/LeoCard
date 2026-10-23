use super::{ScrollContentHeight, Scrollbar, ScrollbarSkin};
use bevy::prelude::*;

pub(super) struct ScrollGeometry {
    pub(super) maximum: f32,
    pub(super) thumb: f32,
    pub(super) travel: f32,
    pub(super) offset: f32,
    pub(super) start: f32,
}

impl ScrollGeometry {
    fn new(
        scroll: &ComputedNode,
        position: &ScrollPosition,
        track: &ComputedNode,
        content_height: Option<&ScrollContentHeight>,
    ) -> Self {
        let height = track.size().y * track.inverse_scale_factor();
        let content = content_height.map_or_else(
            || scroll.content_size().y * scroll.inverse_scale_factor(),
            |height| height.0,
        );
        let visible = scroll.size().y * scroll.inverse_scale_factor();
        let maximum = (content - visible).max(0.0);
        let thumb = (height * (visible / content.max(1.0)).min(1.0))
            .max(32.0)
            .min(height);
        let travel = (height - thumb).max(0.0);
        let offset = if maximum > 0.0 {
            travel * position.y.clamp(0.0, maximum) / maximum
        } else {
            0.0
        };
        Self {
            maximum,
            thumb,
            travel,
            offset,
            start: 0.0,
        }
    }
}

impl Scrollbar {
    pub(super) fn geometry(
        &self,
        scroll: &ComputedNode,
        position: &ScrollPosition,
        track: &ComputedNode,
        content_height: Option<&ScrollContentHeight>,
    ) -> ScrollGeometry {
        let mut geometry = ScrollGeometry::new(scroll, position, track, content_height);
        if matches!(self.skin, ScrollbarSkin::Textured(_)) {
            let inner_height = (track.size().y * track.inverse_scale_factor() - 8.0).max(0.0);
            geometry.start = 6.0;
            geometry.thumb = 32.0_f32.min(inner_height);
            geometry.travel = (inner_height - geometry.thumb).max(0.0);
            geometry.offset = geometry.start
                + if geometry.maximum > 0.0 {
                    geometry.travel * position.y.clamp(0.0, geometry.maximum) / geometry.maximum
                } else {
                    0.0
                };
        }
        geometry
    }
}
