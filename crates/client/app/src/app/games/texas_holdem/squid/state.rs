use super::super::texas_portrait_position;
use crate::app::presentation::table_random_signed;
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, SeatId, TABLE_SEAT_COUNT, TexasHoldemSnapshot};

#[derive(Default)]
pub(crate) struct SquidTableState {
    key: Option<(MatchId, PlayerId)>,
    round: u32,
    pub(super) markers: Vec<Marker>,
}

pub(super) struct Marker {
    pub owner: Option<PlayerId>,
    /// z is the fraction of parent height; the local portrait is bottom anchored.
    pub position: Vec3,
    pub rotation: f32,
    start: Vec3,
    target: Vec3,
    elapsed: f32,
    delay: f32,
}

impl SquidTableState {
    pub(crate) fn observe(&mut self, game: &TexasHoldemSnapshot) {
        let reset = self.key != Some((game.match_id, game.you))
            || self.markers.len() != game.squid.markers.len();
        let new_round = self.round != game.squid.round;
        let seed = u64::from_le_bytes(game.match_id.0[..8].try_into().expect("match ID"))
            ^ u64::from(game.squid.round).wrapping_mul(53);
        if reset {
            self.markers.clear();
        }
        for (index, &owner) in game.squid.markers.iter().enumerate() {
            let position = marker_position(game, index, owner, seed);
            if reset {
                self.markers.push(Marker {
                    owner,
                    position,
                    rotation: table_random_signed(index as u64, seed) * 0.26,
                    start: position,
                    target: position,
                    elapsed: 1.0,
                    delay: 0.0,
                });
            } else {
                let marker = &mut self.markers[index];
                if new_round || marker.owner != owner || marker.target != position {
                    marker.start = marker.position;
                    marker.target = position;
                    marker.elapsed = 0.0;
                    marker.delay = if !new_round && marker.owner != owner {
                        0.45
                    } else {
                        0.0
                    };
                    marker.owner = owner;
                }
            }
        }
        self.key = Some((game.match_id, game.you));
        self.round = game.squid.round;
    }

    pub(crate) fn tick(&mut self, delta: f32) {
        for marker in &mut self.markers {
            marker.elapsed += delta;
            let progress = ((marker.elapsed - marker.delay) / 0.62).clamp(0.0, 1.0);
            marker.position = marker
                .start
                .lerp(marker.target, progress * progress * (3.0 - 2.0 * progress));
        }
    }
}

fn marker_position(
    game: &TexasHoldemSnapshot,
    index: usize,
    owner: Option<PlayerId>,
    seed: u64,
) -> Vec3 {
    let random_x = table_random_signed(index as u64, seed);
    let random_y = table_random_signed(index as u64, seed.wrapping_add(17));
    let Some(owner) = owner else {
        let count = game
            .squid
            .markers
            .iter()
            .filter(|owner| owner.is_none())
            .count();
        let slot = game.squid.markers[..index]
            .iter()
            .filter(|owner| owner.is_none())
            .count();
        let columns = count.min(3);
        let row = slot / columns;
        let row_count = (count - row * columns).min(columns);
        // The draw-pile/street-label column is centred at x=475, below the board.
        return Vec3::new(
            475.0 - 17.0 + (slot % columns) as f32 * 38.0
                - row_count.saturating_sub(1) as f32 * 19.0
                + random_x * 3.0,
            254.0 + row as f32 * 38.0 + random_y * 4.0,
            0.0,
        );
    };
    let origin = game
        .players
        .iter()
        .find(|p| p.id == game.you)
        .map_or(SeatId(0), |p| p.seat);
    let relative = game.players.iter().find(|p| p.id == owner).map_or(0, |p| {
        (p.seat.0 + TABLE_SEAT_COUNT - origin.0) % TABLE_SEAT_COUNT
    });
    let slot = game.squid.markers[..index]
        .iter()
        .filter(|&&p| p == Some(owner))
        .count();
    let mut offset = Vec3::new(
        (slot % 2) as f32 * 16.0 + random_x * 4.0,
        (slot / 2) as f32 * 12.0 + random_y * 4.0,
        0.0,
    );
    let anchor = if relative == 0 {
        offset.y = -offset.y;
        Vec3::new(42.0, -142.0, 1.0)
    } else {
        let portrait = texas_portrait_position(relative);
        let left = if relative <= 2 {
            portrait.x - 38.0
        } else {
            portrait.x + 108.0
        };
        Vec3::new(left, portrait.y + 7.0, 0.0)
    };
    // All coordinates stay inside the 1280-wide design canvas.
    let mut point = anchor + offset;
    point.x = point.x.clamp(3.0, 1244.0);
    point
}
