use super::{ChipZoneLayout, TEXAS_PORTRAIT_HEIGHT, TEXAS_PORTRAIT_WIDTH};
use crate::app::shell::SeatSide;
use bevy::prelude::Vec2;
use leocard_protocol::{PlayerId, SeatId, TexasHoldemSnapshot};
use leocard_texas_holdem::TexasHoldemRuleSet;

/// Keep the original six-seat table for smaller games; expand only the side columns.
#[derive(Clone, Copy)]
pub(super) struct TexasTableLayout {
    player_count: u8,
}

impl TexasTableLayout {
    pub(super) const fn new(player_count: u8) -> Self {
        Self { player_count }
    }

    pub(super) fn for_game(game: &TexasHoldemSnapshot) -> Self {
        Self::new(game.players.len() as u8)
    }

    pub(super) fn player_slot(self, game: &TexasHoldemSnapshot, player: PlayerId) -> u8 {
        let origin = game
            .players
            .iter()
            .find(|p| p.id == game.you)
            .or_else(|| game.players.first())
            .map_or(SeatId(0), |p| p.seat);
        let target = game
            .players
            .iter()
            .find(|p| p.id == player)
            .map_or(origin, |p| p.seat);
        self.relative_slot(origin, target, game.players.iter().map(|p| p.seat))
    }

    pub(super) fn relative_slot(
        self,
        origin: SeatId,
        target: SeatId,
        seats: impl Iterator<Item = SeatId>,
    ) -> u8 {
        let distance = |seat: SeatId| {
            (seat.0 + TexasHoldemRuleSet::MAX_PLAYERS - origin.0) % TexasHoldemRuleSet::MAX_PLAYERS
        };
        let rank = seats
            .filter(|seat| distance(*seat) < distance(target))
            .count() as u8;
        let slots = if self.player_count <= 6 { 6 } else { 8 };
        // Rounding distributes occupied seats symmetrically, even with gaps in room seat IDs.
        ((u16::from(rank) * slots + u16::from(self.player_count) / 2)
            / u16::from(self.player_count.max(1))) as u8
    }

    pub(super) fn side(self, slot: u8) -> SeatSide {
        let top = if self.player_count <= 6 { 3 } else { 4 };
        if slot < top {
            SeatSide::Left
        } else if slot == top {
            SeatSide::Top
        } else {
            SeatSide::Right
        }
    }

    pub(super) fn chip_zone(self, slot: u8) -> ChipZoneLayout {
        let (center_x, top) = if self.player_count <= 6 {
            match slot {
                0 => (640.0, 405.0),
                1 => (282.0, 346.0),
                2 => (282.0, 123.0),
                3 => (640.0, 23.0),
                4 => (998.0, 123.0),
                _ => (998.0, 346.0),
            }
        } else {
            match slot {
                0 => (640.0, 405.0),
                1 => (258.0, 346.0),
                2 => (258.0, 201.0),
                3 => (258.0, 56.0),
                4 => (640.0, 23.0),
                5 => (1022.0, 56.0),
                6 => (1022.0, 201.0),
                _ => (1022.0, 346.0),
            }
        };
        // Extend the former 116-high top zone upwards, retaining its bottom edge.
        ChipZoneLayout {
            left: center_x - 105.0,
            top,
            width: 210.0,
            height: 130.0,
        }
    }

    pub(super) fn portrait_position(self, slot: u8) -> Vec2 {
        let zone = self.chip_zone(slot);
        let side = self.side(slot);
        let left = match side {
            SeatSide::Left => zone.left - TEXAS_PORTRAIT_WIDTH - 8.0,
            SeatSide::Top | SeatSide::Right => zone.left + zone.width + 8.0,
        };
        let top = if matches!(side, SeatSide::Top) {
            // Original six-seat portrait position, independent of the equalized chip-zone height.
            37.0 + (116.0 - TEXAS_PORTRAIT_HEIGHT) * 0.5 - 16.0
        } else {
            zone.top + (zone.height - TEXAS_PORTRAIT_HEIGHT) * 0.5
        };
        Vec2::new(left, top)
    }
}
