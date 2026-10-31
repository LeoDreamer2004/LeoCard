use super::{ChipZoneLayout, TEXAS_PORTRAIT_HEIGHT, TEXAS_PORTRAIT_WIDTH};
use crate::app::presentation::{SeatSide, TableSeat, TableSeatAnchors, TableSeats};
use bevy::prelude::Vec2;
use leocard_game_common::PlayerId;
use leocard_protocol::{SeatId, TexasSnapshot};
use leocard_texas_holdem::TexasRuleSet;

/// Seat mapping and shared geometry for the desktop and compact Android tables.
#[derive(Clone, Copy)]
pub(super) struct TexasTableLayout {
    player_count: u8,
}

impl TexasTableLayout {
    pub(super) const BOARD_CARD_SIZE: Vec2 = if cfg!(target_os = "android") {
        Vec2::new(84.0, 114.0)
    } else {
        Vec2::new(56.0, 76.0)
    };
    pub(super) const BOARD_TOP: f32 = if cfg!(target_os = "android") {
        142.0
    } else {
        160.0
    };
    pub(super) const BOARD_ROW_HEIGHT: f32 = if cfg!(target_os = "android") {
        126.0
    } else {
        90.0
    };
    pub(super) const HAND_SCALE: f32 = if cfg!(target_os = "android") {
        1.4
    } else {
        1.0
    };
    pub(super) const ACTION_SCALE: f32 = if cfg!(target_os = "android") {
        1.4
    } else {
        1.0
    };
    pub(super) const ACTION_BOTTOM: f32 = if cfg!(target_os = "android") {
        177.0
    } else {
        124.0
    };

    pub(super) fn board_card_origin(index: usize) -> Vec2 {
        let content_width = 46.0 + 8.0 + 5.0 * Self::BOARD_CARD_SIZE.x + 6.0 * 7.0;
        Vec2::new(
            640.0 - content_width * 0.5
                + 46.0
                + 8.0
                + 14.0
                + index as f32 * (Self::BOARD_CARD_SIZE.x + 7.0),
            Self::BOARD_TOP + (Self::BOARD_ROW_HEIGHT - Self::BOARD_CARD_SIZE.y) * 0.5,
        )
    }

    pub(super) fn draw_pile_center() -> Vec2 {
        let first = Self::board_card_origin(0);
        Vec2::new(
            first.x - 45.0,
            Self::BOARD_TOP + Self::BOARD_ROW_HEIGHT * 0.5 - 12.5,
        )
    }

    pub(super) fn initial_deal_source() -> Vec2 {
        if cfg!(target_os = "android") {
            Self::draw_pile_center() - Vec2::new(18.0, 24.5)
        } else {
            Vec2::new(456.0, 212.0)
        }
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
        let width = if cfg!(target_os = "android")
            && matches!(seat, TableSeat::Left(_) | TableSeat::Right(_))
        {
            194.0
        } else {
            210.0
        };
        let position = seat.position(TableSeatAnchors {
            bottom: Vec2::new(
                640.0,
                if cfg!(target_os = "android") {
                    350.0
                } else {
                    405.0
                },
            ),
            top: Vec2::new(640.0, 23.0),
            columns: if cfg!(target_os = "android") {
                Vec2::new(224.0, 1056.0)
            } else if expanded {
                Vec2::new(258.0, 1022.0)
            } else {
                Vec2::new(282.0, 998.0)
            },
            rows: if cfg!(target_os = "android") && expanded {
                // Keep all three portrait/stack rows clear of the enlarged mobile header.
                &[356.0, 228.0, 100.0]
            } else if cfg!(target_os = "android") {
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
            height: if cfg!(target_os = "android") {
                96.0
            } else {
                130.0
            },
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
