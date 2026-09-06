use super::*;
use leocard_protocol::{PlayerId, ShengjiSnapshot};

pub(super) fn shengji_relative_seat(game: &ShengjiSnapshot, player: PlayerId) -> Option<u8> {
    const SEAT_COUNT: u8 = 4;
    let own_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == game.you)?
        .seat
        .0;
    let player_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == player)?
        .seat
        .0;
    Some((player_seat + SEAT_COUNT - own_seat) % SEAT_COUNT)
}

/// 百分比坐标以整张牌桌为基准，并落在各座位靠桌面内侧的出牌区。
pub(super) fn shengji_seat_route_anchor(relative_seat: u8) -> Vec2 {
    match relative_seat {
        0 => Vec2::new(50.0, 88.0),
        1 => Vec2::new(20.0, 52.0),
        2 => Vec2::new(50.0, 20.0),
        3 => Vec2::new(80.0, 52.0),
        _ => Vec2::new(50.0, 50.0),
    }
}

pub(super) fn shengji_player_route_anchor(
    game: &ShengjiSnapshot,
    player: PlayerId,
) -> Option<Vec2> {
    shengji_relative_seat(game, player).map(shengji_seat_route_anchor)
}

/// 扣底的“查找同牌”动画要指向玩家本人，而不是平时的出牌区。
/// 这组锚点因此落在四个玩家框的头像附近。
pub(super) fn shengji_seat_player_anchor(relative_seat: u8) -> Vec2 {
    match relative_seat {
        0 => Vec2::new(92.0, 91.0),
        1 => Vec2::new(4.5, 52.0),
        2 => Vec2::new(50.0, 10.5),
        3 => Vec2::new(95.5, 52.0),
        _ => Vec2::new(50.0, 50.0),
    }
}

pub(super) fn shengji_player_panel_anchor(
    game: &ShengjiSnapshot,
    player: PlayerId,
) -> Option<Vec2> {
    shengji_relative_seat(game, player).map(shengji_seat_player_anchor)
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
                    start: shengji_seat_route_anchor(relative),
                    end: shengji_seat_route_anchor((relative + 2) % 4),
                    packet_count: 5,
                })
            })
            .collect(),
        ShengjiPresentationKind::CrossingReturned { player, .. } => {
            shengji_relative_seat(game, *player)
                .map(|relative| ShengjiPresentationRoute {
                    start: shengji_seat_route_anchor(relative),
                    end: shengji_seat_route_anchor((relative + 2) % 4),
                    packet_count: 5,
                })
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}
