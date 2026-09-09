use super::{
    ShengjiAudioCue, ShengjiPlayPresentationKind, ShengjiPresentationKind, ShengjiSoundKind,
    shengji_partner_player,
};
use crate::app::presentation::ACCENT;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, ShengjiSnapshot};
use leocard_shengji::{Component, ShengjiBidTrump, ShengjiClassifiedPlay, ShengjiRank};
use leocard_shengji::{ShengjiCard, ShengjiSuit};

pub(super) fn classify_play_presentation(
    play: &ShengjiClassifiedPlay,
) -> ShengjiPlayPresentationKind {
    if play.is_throw() {
        return ShengjiPlayPresentationKind::Throw;
    }
    match play.strongest_component() {
        Component::Single { .. } => ShengjiPlayPresentationKind::Single,
        Component::Pair { .. } => ShengjiPlayPresentationKind::Pair,
        Component::Tractor { .. } => ShengjiPlayPresentationKind::Tractor,
        Component::Triple { .. } => ShengjiPlayPresentationKind::Triple,
        Component::Titanic { .. } => ShengjiPlayPresentationKind::Titanic,
        Component::Quad { .. } => ShengjiPlayPresentationKind::Bomb,
        Component::Spaceship { .. } => ShengjiPlayPresentationKind::Spaceship,
    }
}

pub(super) fn should_show_play_presentation(
    kind: ShengjiPlayPresentationKind,
    is_lead: bool,
    throw_penalty: u16,
) -> bool {
    if throw_penalty > 0
        || matches!(
            kind,
            ShengjiPlayPresentationKind::Single
                | ShengjiPlayPresentationKind::Pair
                | ShengjiPlayPresentationKind::Triple
        )
    {
        return false;
    }
    kind != ShengjiPlayPresentationKind::Throw || is_lead
}

pub(super) fn queue_play_audio(
    cues: &mut Vec<ShengjiAudioCue>,
    kind: ShengjiPlayPresentationKind,
    seed: u64,
    start: f32,
) {
    let cue = |kind, remaining, volume, offset| {
        ShengjiAudioCue::new(
            kind,
            start + remaining,
            volume,
            seed.wrapping_mul(31) + offset,
        )
    };
    match kind {
        ShengjiPlayPresentationKind::Single => {
            cues.push(cue(ShengjiSoundKind::CardPlace, 0.0, 0.34, 1));
        }
        ShengjiPlayPresentationKind::Pair => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.30, 1),
            cue(ShengjiSoundKind::CardPlace, 0.075, 0.42, 2),
        ]),
        ShengjiPlayPresentationKind::Tractor => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.44, 1),
            cue(ShengjiSoundKind::CardPlace, 0.18, 0.32, 2),
        ]),
        ShengjiPlayPresentationKind::Triple => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.26, 1),
            cue(ShengjiSoundKind::CardPlace, 0.065, 0.32, 2),
            cue(ShengjiSoundKind::CardPlace, 0.13, 0.44, 3),
        ]),
        ShengjiPlayPresentationKind::Titanic => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.46, 1),
            cue(ShengjiSoundKind::Heavy, 0.16, 0.48, 2),
        ]),
        ShengjiPlayPresentationKind::Bomb => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.32, 1),
            cue(ShengjiSoundKind::Bomb, 0.10, 0.52, 2),
        ]),
        ShengjiPlayPresentationKind::Spaceship => cues.extend([
            cue(ShengjiSoundKind::Bomb, 0.0, 0.48, 1),
            cue(ShengjiSoundKind::Bomb, 0.18, 0.44, 2),
            cue(ShengjiSoundKind::CardShove, 0.28, 0.50, 3),
        ]),
        ShengjiPlayPresentationKind::Throw => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.46, 1),
            cue(ShengjiSoundKind::CardPlace, 0.20, 0.30, 2),
        ]),
    }
}

pub(super) fn presentation_text(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> (String, String) {
    let player_name = |player: PlayerId| {
        game.players
            .iter()
            .find(|candidate| candidate.id == player)
            .map_or_else(
                || format!("玩家{}", player.0 + 1),
                |player| player.name.clone(),
            )
    };
    match kind {
        ShengjiPresentationKind::Declaration {
            player,
            trump,
            label,
        } => (
            (*label).to_owned(),
            format!("{} · {}", player_name(*player), bid_trump_label(*trump)),
        ),
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
        ShengjiPresentationKind::BottomCopy {
            from_player,
            player,
            trump,
            count,
        } => (
            format!("第{count}次抄底"),
            from_player.map_or_else(
                || format!("{} · {}", player_name(*player), bid_trump_label(*trump)),
                |from_player| {
                    format!(
                        "{} → {} · {}",
                        player_name(from_player),
                        player_name(*player),
                        bid_trump_label(*trump)
                    )
                },
            ),
        ),
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
        ShengjiPresentationKind::TrumpKill { covered, .. } => (
            if *covered { "盖毙" } else { "毙牌" }.to_owned(),
            String::new(),
        ),
        ShengjiPresentationKind::Play { kind, .. } => (kind.label().to_owned(), String::new()),
    }
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
        ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Bomb,
            ..
        }
        | ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Spaceship,
            ..
        } => Color::srgb(1.0, 0.27, 0.16),
        ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Titanic,
            ..
        } => Color::srgb(0.24, 0.68, 1.0),
        _ => ACCENT,
    }
}

fn bid_trump_label(trump: ShengjiBidTrump) -> String {
    match trump {
        ShengjiBidTrump::Suit(ShengjiSuit::Diamond) => "♦ 方块主",
        ShengjiBidTrump::Suit(ShengjiSuit::Club) => "♣ 梅花主",
        ShengjiBidTrump::Suit(ShengjiSuit::Heart) => "♥ 红桃主",
        ShengjiBidTrump::Suit(ShengjiSuit::Spade) => "♠ 黑桃主",
        ShengjiBidTrump::NoTrumpSmallJoker | ShengjiBidTrump::NoTrumpBigJoker => "无主",
    }
    .to_owned()
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
