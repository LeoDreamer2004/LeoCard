use super::{ChipZoneLayout, TEXAS_PORTRAIT_HEIGHT, TEXAS_PORTRAIT_WIDTH};
use crate::app::presentation::{
    SeatSide, TableSeat, TableSeatAnchors, TableSeats, table_card_pile_top_center,
};
use bevy::prelude::Vec2;
use leocard_game_common::PlayerId;
use leocard_platform::is_mobile;
use leocard_protocol::{SeatId, TexasSnapshot};
use leocard_texas_holdem::TexasRuleSet;

/// Seat mapping and shared geometry for the desktop and compact Android tables.
#[derive(Clone, Copy)]
pub(super) struct TexasTableLayout {
    player_count: u8,
}

impl TexasTableLayout {
    pub(super) fn board_card_size() -> Vec2 {
        if is_mobile() {
            Vec2::new(92.0, 122.0)
        } else {
            Vec2::new(62.0, 82.0)
        }
    }

    pub(super) fn board_top() -> f32 {
        if is_mobile() { 154.0 } else { 180.0 }
    }

    pub(super) fn board_row_height() -> f32 {
        if is_mobile() { 140.0 } else { 100.0 }
    }

    pub(super) fn squid_pool_anchor_y() -> f32 {
        if is_mobile() { 304.0 } else { 318.0 }
    }

    pub(super) fn local_bet_top() -> f32 {
        if is_mobile() { 350.0 } else { 405.0 }
    }

    pub(super) fn hand_scale() -> f32 {
        if is_mobile() { 1.4 } else { 1.0 }
    }

    pub(super) fn action_scale() -> f32 {
        if is_mobile() { 1.4 } else { 1.0 }
    }

    pub(super) fn action_bottom() -> f32 {
        if is_mobile() { 177.0 } else { 124.0 }
    }

    pub(super) fn board_width() -> f32 {
        54.0 + 8.0 + 5.0 * Self::board_card_size().x + 6.0 * 7.0
    }

    pub(super) fn board_card_origin(index: usize) -> Vec2 {
        Vec2::new(
            640.0 - Self::board_width() * 0.5
                + 54.0
                + 8.0
                + 14.0
                + index as f32 * (Self::board_card_size().x + 7.0),
            Self::board_top() + (Self::board_row_height() - Self::board_card_size().y) * 0.5,
        )
    }

    pub(super) fn draw_pile_center() -> Vec2 {
        // The 50px pile is centred in a 54px column, 8px below the row top.
        Vec2::new(
            640.0 - Self::board_width() * 0.5 + 2.0,
            Self::board_top() + 8.0,
        ) + table_card_pile_top_center()
    }

    pub(super) fn initial_deal_source() -> Vec2 {
        Self::draw_pile_center() - Vec2::new(18.0, 24.5)
    }

    pub(super) const fn new(player_count: u8) -> Self {
        Self { player_count }
    }

    pub(super) fn for_game(game: &TexasSnapshot) -> Self {
        Self::new(game.players.len() as u8)
    }

    pub(super) fn player_slot(self, game: &TexasSnapshot, player: PlayerId) -> u8 {
        let origin = game
            .players
            .iter()
            .find(|p| p.player.id == game.you)
            .or_else(|| game.players.first())
            .map_or(SeatId(0), |p| p.seat);
        let target = game
            .players
            .iter()
            .find(|p| p.player.id == player)
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
            TexasRuleSet::MAX_PLAYERS,
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
        let seat = self.seats().seat(slot);
        let width = if is_mobile() && matches!(seat, TableSeat::Left(_) | TableSeat::Right(_)) {
            194.0
        } else {
            210.0
        };
        let position = seat.position(TableSeatAnchors {
            bottom: Vec2::new(640.0, Self::local_bet_top()),
            top: Vec2::new(640.0, 23.0),
            columns: if is_mobile() {
                Vec2::new(224.0, 1056.0)
            } else if expanded {
                Vec2::new(258.0, 1022.0)
            } else {
                Vec2::new(282.0, 998.0)
            },
            rows: if is_mobile() && expanded {
                // Keep all three portrait/stack rows clear of the enlarged mobile header.
                &[356.0, 228.0, 100.0]
            } else if is_mobile() {
                &[346.0, 116.0]
            } else if expanded {
                &[346.0, 201.0, 56.0]
            } else {
                &[346.0, 123.0]
            },
        });
        ChipZoneLayout {
            left: position.x - width * 0.5,
            top: position.y,
            width,
            height: if is_mobile() { 96.0 } else { 130.0 },
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
