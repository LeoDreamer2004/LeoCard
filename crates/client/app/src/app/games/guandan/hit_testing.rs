use bevy::prelude::*;

pub(super) struct CardHitRegion {
    pub index: usize,
    pub rect: Rect,
}

/// Follow the visible top card along the whole stroke, including fast moves
/// between frames. A covered card must not be selected just because its full
/// rectangle also intersects the pointer's path.
pub(super) fn crossed_cards(start: Vec2, end: Vec2, regions: &[CardHitRegion]) -> Vec<usize> {
    let mut boundaries = vec![0.0, 1.0];
    for region in regions {
        if let Some((enter, leave)) = segment_interval(start, end, region.rect) {
            boundaries.extend([enter, leave]);
        }
    }
    boundaries.sort_by(f32::total_cmp);
    boundaries.dedup_by(|left, right| (*left - *right).abs() < 0.000_001);
    let mut samples = boundaries.clone();
    samples.extend(boundaries.windows(2).map(|pair| (pair[0] + pair[1]) * 0.5));
    samples.sort_by(f32::total_cmp);
    let mut crossed = Vec::new();
    for fraction in samples {
        let point = start.lerp(end, fraction);
        if let Some(region) = regions
            .iter()
            .filter(|region| region.rect.contains(point))
            .max_by_key(|region| region.index)
            && crossed.last() != Some(&region.index)
        {
            crossed.push(region.index);
        }
    }
    crossed
}

fn segment_interval(start: Vec2, end: Vec2, rect: Rect) -> Option<(f32, f32)> {
    let delta = end - start;
    let mut enter: f32 = 0.0;
    let mut leave: f32 = 1.0;
    for axis in 0..2 {
        if delta[axis].abs() < f32::EPSILON {
            if start[axis] < rect.min[axis] || start[axis] > rect.max[axis] {
                return None;
            }
            continue;
        }
        let first = (rect.min[axis] - start[axis]) / delta[axis];
        let last = (rect.max[axis] - start[axis]) / delta[axis];
        enter = enter.max(first.min(last));
        leave = leave.min(first.max(last));
        if enter > leave {
            return None;
        }
    }
    Some((enter, leave))
}
