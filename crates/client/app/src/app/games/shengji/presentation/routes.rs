use super::{ShengjiPresentationKind, ShengjiPresentationRoute};
use crate::app::presentation::{TableSeatAnchors, TableSeats};
use bevy::prelude::*;
use leocard_protocol::{PlayerId, ShengjiSnapshot};

fn shengji_relative_seat(game: &ShengjiSnapshot, player: PlayerId) -> Option<u8> {
    let seat_count = game.rules.player_count();
    let own_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == game.you)
        .map_or(0, |player| player.seat.0);
    let player_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == player)?
        .seat
        .0;
    Some(TableSeats::new(seat_count).relative(own_seat, player_seat))
}

/// 百分比坐标以整张牌桌为基准，发牌与抄底共用人数布局。
fn route_anchor(relative: u8, count: u8) -> Vec2 {
    TableSeats::new(count)
        .seat(relative)
        .position(TableSeatAnchors {
            bottom: Vec2::new(50.0, 88.0),
            top: Vec2::new(50.0, 20.0),
            columns: Vec2::new(20.0, 80.0),
            rows: if count == 6 { &[72.0, 32.0] } else { &[52.0] },
        })
}

pub(in super::super) fn shengji_player_route_anchor(
    game: &ShengjiSnapshot,
    player: PlayerId,
) -> Option<Vec2> {
    shengji_relative_seat(game, player)
        .map(|relative| route_anchor(relative, game.rules.player_count()))
}

/// 扳底查找同牌指向头像而非出牌区，仍按相同的座位方向取点。
pub(super) fn shengji_player_panel_anchor(
    game: &ShengjiSnapshot,
    player: PlayerId,
) -> Option<Vec2> {
    shengji_relative_seat(game, player).map(|relative| {
        let count = game.rules.player_count();
        TableSeats::new(count)
            .seat(relative)
            .position(TableSeatAnchors {
                bottom: Vec2::new(92.0, 91.0),
                top: Vec2::new(50.0, 10.5),
                columns: Vec2::new(4.5, 95.5),
                rows: if count == 6 { &[72.0, 32.0] } else { &[52.0] },
            })
    })
}

pub(super) fn shengji_power_outage_anchors(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> Option<(Vec2, Vec2)> {
    let ShengjiPresentationKind::PowerOutage {
        from_dealer,
        dealer,
        ..
    } = kind
    else {
        return None;
    };
    let start = from_dealer
        .and_then(|player| shengji_player_route_anchor(game, player))
        .unwrap_or(Vec2::new(50.0, 50.0));
    let end = shengji_player_route_anchor(game, *dealer).unwrap_or(Vec2::new(50.0, 50.0));
    Some((start, end))
}

pub(super) fn shengji_partner_player(game: &ShengjiSnapshot, player: PlayerId) -> Option<PlayerId> {
    let seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == player)?
        .seat
        .0;
    game.players
        .iter()
        .find(|candidate| candidate.seat.0 == (seat + 2) % 4)
        .map(|candidate| candidate.id)
}

pub(super) fn shengji_presentation_routes(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> Vec<ShengjiPresentationRoute> {
    match kind {
        ShengjiPresentationKind::BottomCopy {
            from_player,
            player,
            ..
        } => shengji_player_route_anchor(game, *player)
            .map(|end| ShengjiPresentationRoute {
                start: from_player
                    .and_then(|player| shengji_player_route_anchor(game, player))
                    .unwrap_or(Vec2::new(50.0, 50.0)),
                end,
                packet_count: game.rules.kitty_size(),
            })
            .into_iter()
            .collect(),
        ShengjiPresentationKind::CrossingStarted { players } => players
            .iter()
            .filter_map(|player| {
                let relative = shengji_relative_seat(game, *player)?;
                Some(ShengjiPresentationRoute {
                    start: route_anchor(relative, 4),
                    end: route_anchor((relative + 2) % 4, 4),
                    packet_count: 5,
                })
            })
            .collect(),
        ShengjiPresentationKind::CrossingReturned { player, .. } => {
            shengji_relative_seat(game, *player)
                .map(|relative| ShengjiPresentationRoute {
                    start: route_anchor(relative, 4),
                    end: route_anchor((relative + 2) % 4, 4),
                    packet_count: 5,
                })
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}
