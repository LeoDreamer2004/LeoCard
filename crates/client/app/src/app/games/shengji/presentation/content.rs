use super::routes::shengji_partner_player;

use super::ShengjiPresentationKind;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, ShengjiSnapshot};
use leocard_shengji::{ShengjiCard, ShengjiRank, ShengjiSuit};

pub(super) fn presentation_text(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> Option<(String, String)> {
    let player_name = |player: PlayerId| {
        game.players
            .iter()
            .find(|candidate| candidate.id == player)
            .map_or_else(
                || format!("玩家{}", player.0 + 1),
                |player| player.name.clone(),
            )
    };
    Some(match kind {
        ShengjiPresentationKind::PowerOutage {
            from_dealer,
            dealer,
            level,
        } => (
            "断电换庄".to_owned(),
            from_dealer.map_or_else(
                || format!("{} 接庄 · 打{}", player_name(*dealer), rank_label(*level)),
                |from_dealer| {
                    format!(
                        "{} → {} · 打{}",
                        player_name(from_dealer),
                        player_name(*dealer),
                        rank_label(*level)
                    )
                },
            ),
        ),
        ShengjiPresentationKind::BottomFlip {
            card,
            matches,
            dealer,
        } => (
            "扳底翻牌".to_owned(),
            dealer.map_or_else(
                || format!("{} 人持有同牌，继续判定", matches.len()),
                |dealer| {
                    format!(
                        "{} 坐庄 · 翻出{}",
                        player_name(dealer),
                        card_face_label(*card)
                    )
                },
            ),
        ),
        ShengjiPresentationKind::BottomCopy { .. } => return None,
        ShengjiPresentationKind::CrossingStarted { players } => {
            let directions = players
                .iter()
                .filter_map(|player| {
                    shengji_partner_player(game, *player).map(|partner| {
                        format!("{} → {}", player_name(*player), player_name(partner))
                    })
                })
                .collect::<Vec<_>>()
                .join("　");
            (
                "五主过江".to_owned(),
                if directions.is_empty() {
                    format!("{} 名玩家向对家交牌", players.len())
                } else {
                    directions
                },
            )
        }
        ShengjiPresentationKind::CrossingReturned { player, complete } => (
            if *complete {
                "过江完成".to_owned()
            } else {
                "归还五张".to_owned()
            },
            shengji_partner_player(game, *player).map_or_else(
                || format!("{} 已完成归还", player_name(*player)),
                |partner| format!("{} → {}", player_name(*player), player_name(partner)),
            ),
        ),
        ShengjiPresentationKind::TrumpKill { .. } => return None,
    })
}

pub(super) fn presentation_color(kind: &ShengjiPresentationKind) -> Color {
    match kind {
        ShengjiPresentationKind::PowerOutage { .. } => Color::srgb(0.35, 0.78, 1.0),
        ShengjiPresentationKind::BottomFlip { .. } => Color::srgb(1.0, 0.70, 0.16),
        ShengjiPresentationKind::BottomCopy { .. } => Color::srgb(0.82, 0.43, 1.0),
        ShengjiPresentationKind::CrossingStarted { .. }
        | ShengjiPresentationKind::CrossingReturned { .. } => Color::srgb(0.20, 0.88, 0.72),
        ShengjiPresentationKind::TrumpKill { covered: true, .. } => Color::srgb(1.0, 0.62, 0.14),
        ShengjiPresentationKind::TrumpKill { covered: false, .. } => Color::srgb(0.36, 0.94, 0.67),
    }
}

pub(super) fn rank_label(rank: ShengjiRank) -> &'static str {
    match rank {
        ShengjiRank::Two => "2",
        ShengjiRank::Three => "3",
        ShengjiRank::Four => "4",
        ShengjiRank::Five => "5",
        ShengjiRank::Six => "6",
        ShengjiRank::Seven => "7",
        ShengjiRank::Eight => "8",
        ShengjiRank::Nine => "9",
        ShengjiRank::Ten => "10",
        ShengjiRank::Jack => "J",
        ShengjiRank::Queen => "Q",
        ShengjiRank::King => "K",
        ShengjiRank::Ace => "A",
        ShengjiRank::SmallJoker => "小王",
        ShengjiRank::BigJoker => "大王",
    }
}

fn card_face_label(card: ShengjiCard) -> String {
    let suit = match card.suit() {
        Some(ShengjiSuit::Diamond) => "♦",
        Some(ShengjiSuit::Club) => "♣",
        Some(ShengjiSuit::Heart) => "♥",
        Some(ShengjiSuit::Spade) => "♠",
        None => "",
    };
    format!("{suit}{}", rank_label(card.rank()))
}
