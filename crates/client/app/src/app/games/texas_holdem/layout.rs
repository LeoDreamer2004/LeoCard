use super::{ChipZoneLayout, TEXAS_PORTRAIT_HEIGHT, TEXAS_PORTRAIT_WIDTH};
use crate::app::presentation::{SeatSide, TableSeatAnchors, TableSeats};
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
        self.seats().occupied_slot(
            origin.0,
            target.0,
            seats.map(|seat| seat.0),
            TexasHoldemRuleSet::MAX_PLAYERS,
        )
    }

    fn seats(self) -> TableSeats {
        TableSeats::new(if self.player_count <= 6 { 6 } else { 8 })
    }

    pub(super) fn side(self, slot: u8) -> SeatSide {
        if slot == 0 {
            SeatSide::Left
        } else {
            self.seats().seat(slot).side()
        }
    }

    pub(super) fn chip_zone(self, slot: u8) -> ChipZoneLayout {
        let expanded = self.player_count > 6;
        let position = self.seats().seat(slot).position(TableSeatAnchors {
            bottom: Vec2::new(640.0, 405.0),
            top: Vec2::new(640.0, 23.0),
            columns: if expanded {
                Vec2::new(258.0, 1022.0)
            } else {
                Vec2::new(282.0, 998.0)
            },
            rows: if expanded {
                &[346.0, 201.0, 56.0]
            } else {
                &[346.0, 123.0]
            },
        });
        ChipZoneLayout {
            left: position.x - 105.0,
            top: position.y,
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
