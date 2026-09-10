//! 开发者手牌输入框的显示同步。

use super::{DeveloperHandInput, DeveloperHandInputField, DeveloperHandInputText};
use crate::app::presentation::{ACCENT, BORDER, MUTED, TEXT};
#[cfg(feature = "developer")]
use crate::app::{ClientResource, PageErrorState, game_command};
use bevy::prelude::*;
#[cfg(feature = "developer")]
use leocard_mahjong::{
    MahjongDragon, MahjongHandReplacementError, MahjongSuit, MahjongTileKind, MahjongWind,
};
#[cfg(feature = "developer")]
use leocard_protocol::{MahjongCommand, QiGui523Command};
#[cfg(feature = "developer")]
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};
#[cfg(feature = "developer")]
use std::collections::HashMap;

pub(crate) fn developer_hand_input_label(
    input: &DeveloperHandInput,
    placeholder: &'static str,
) -> String {
    if input.value.is_empty() {
        if input.focused {
            "│".to_owned()
        } else {
            placeholder.to_owned()
        }
    } else {
        format!("{}{}", input.value, if input.focused { "│" } else { "" })
    }
}

pub(crate) fn sync_developer_hand_input_text(
    input: Res<DeveloperHandInput>,
    mut labels: Query<(&DeveloperHandInputText, &mut Text, &mut TextColor)>,
    mut fields: Query<(&mut Node, &mut BorderColor), With<DeveloperHandInputField>>,
) {
    if !input.is_changed() {
        return;
    }
    let expected_color = if input.value.is_empty() { MUTED } else { TEXT };
    for (label, mut text, mut color) in &mut labels {
        let expected = developer_hand_input_label(&input, label.placeholder);
        if text.0 != expected {
            text.0 = expected;
        }
        if color.0 != expected_color {
            color.0 = expected_color;
        }
    }
    for (mut node, mut border) in &mut fields {
        node.border = UiRect::all(px(if input.focused { 2 } else { 1 }));
        border.set_all(if input.focused { ACCENT } else { BORDER });
    }
}

pub(crate) fn append_developer_hand_input(value: &mut String, text: &str) {
    const MAX_DEVELOPER_HAND_INPUT: usize = 192;
    for character in text
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
    {
        if value.len() >= MAX_DEVELOPER_HAND_INPUT {
            break;
        }
        value.push(character.to_ascii_uppercase());
    }
}

#[cfg(feature = "developer")]
pub(crate) fn submit_developer_hand(
    developer_hand: &mut DeveloperHandInput,
    page_error: &mut PageErrorState,
    client: Option<&mut ClientResource>,
) {
    developer_hand.focused = false;
    let input = developer_hand.value.trim().to_owned();
    if input.is_empty() {
        page_error.error = None;
        return;
    }
    let Some(client) = client else {
        return;
    };
    let command = if let Some(game) = client.0.model().mahjong_game() {
        parse_developer_mahjong_hand(&input).and_then(|tiles| {
            if tiles.len() != game.your_hand.len() {
                return Err(MahjongHandReplacementError::WrongTileCount {
                    expected: game.your_hand.len() as u16,
                    actual: tiles.len() as u16,
                }
                .to_string());
            }
            Ok(game_command(MahjongCommand::SetDeveloperHand { tiles }))
        })
    } else {
        parse_developer_hand(&input)
            .map(|cards| game_command(QiGui523Command::SetDeveloperHand { cards }))
    };
    match command {
        Ok(command) => {
            if client.0.send(command) {
                page_error.error = None;
                developer_hand.value.clear();
            } else {
                page_error.error = Some("当前未连接，无法编辑开发者手牌".to_owned());
            }
        }
        Err(error) => page_error.error = Some(error),
    }
}

#[cfg(feature = "developer")]
pub fn parse_developer_hand(input: &str) -> Result<Vec<QiGuiCard>, String> {
    let source = input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if source.is_empty() {
        return Ok(Vec::new());
    }
    if !matches!(source[0], 'S' | 'H' | 'C' | 'D' | 'R' | 'B') {
        return parse_developer_hand_by_rank(&source);
    }
    if !source.len().is_multiple_of(2) {
        return Err("每张牌必须使用两个字符，例如 S4、ST、RJ".to_owned());
    }
    let mut cards = Vec::new();
    let mut copies = HashMap::<(QiGuiSuit, QiGuiRank), u8>::new();
    for (index, code) in source.as_chunks::<2>().0.iter().enumerate() {
        let (suit, rank) = match (code[0], code[1]) {
            ('R', 'J') => (QiGuiSuit::Spade, QiGuiRank::Joker),
            ('B', 'J') => (QiGuiSuit::Club, QiGuiRank::Joker),
            (suit, rank) => {
                let suit = match suit {
                    'S' => QiGuiSuit::Spade,
                    'H' => QiGuiSuit::Heart,
                    'C' => QiGuiSuit::Club,
                    'D' => QiGuiSuit::Diamond,
                    other => {
                        return Err(format!("开发者手牌第 {} 项的花色 {other} 无效", index + 1));
                    }
                };
                let rank = parse_developer_rank(rank)
                    .ok_or_else(|| format!("开发者手牌第 {} 项的点数 {rank} 无效", index + 1))?;
                (suit, rank)
            }
        };
        let copy = copies.entry((suit, rank)).or_default();
        cards.push(QiGuiCard::suited(*copy, suit, rank));
        *copy += 1;
    }
    Ok(cards)
}

#[cfg(feature = "developer")]
fn parse_developer_hand_by_rank(source: &[char]) -> Result<Vec<QiGuiCard>, String> {
    let mut cards = Vec::with_capacity(source.len());
    let mut copies = HashMap::<(QiGuiSuit, QiGuiRank), u8>::new();
    for (index, code) in source.iter().copied().enumerate() {
        let rank = if code == '0' {
            QiGuiRank::Joker
        } else {
            parse_developer_rank(code)
                .ok_or_else(|| format!("开发者手牌第 {} 项的点数 {code} 无效", index + 1))?
        };
        let suits = if rank == QiGuiRank::Joker {
            &[QiGuiSuit::Spade, QiGuiSuit::Club][..]
        } else {
            &QiGuiSuit::IN_STRENGTH_ORDER
        };
        let suit = suits[fastrand::usize(..suits.len())];
        let copy = copies.entry((suit, rank)).or_default();
        cards.push(QiGuiCard::suited(*copy, suit, rank));
        *copy += 1;
    }
    Ok(cards)
}

#[cfg(feature = "developer")]
fn parse_developer_rank(code: char) -> Option<QiGuiRank> {
    Some(match code {
        '2' => QiGuiRank::Two,
        '3' => QiGuiRank::Three,
        '4' => QiGuiRank::Four,
        '5' => QiGuiRank::Five,
        '6' => QiGuiRank::Six,
        '7' => QiGuiRank::Seven,
        '8' => QiGuiRank::Eight,
        '9' => QiGuiRank::Nine,
        'T' => QiGuiRank::Ten,
        'J' => QiGuiRank::Jack,
        'Q' => QiGuiRank::Queen,
        'K' => QiGuiRank::King,
        'A' => QiGuiRank::Ace,
        _ => return None,
    })
}

#[cfg(feature = "developer")]
pub fn parse_developer_mahjong_hand(input: &str) -> Result<Vec<MahjongTileKind>, String> {
    let source = input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if source.is_empty() {
        return Ok(Vec::new());
    }
    let mut digits = Vec::new();
    let mut tiles = Vec::new();
    for code in source {
        if code.is_ascii_digit() {
            digits.push(code);
            continue;
        }
        if digits.is_empty() {
            return Err(format!("花色 {code} 前没有牌面数字"));
        }
        let (suit, maximum) = match code {
            'M' => (Some(MahjongSuit::Characters), 9),
            'P' => (Some(MahjongSuit::Dots), 9),
            'S' => (Some(MahjongSuit::Bamboo), 9),
            'Z' => (None, 7),
            _ => return Err(format!("麻将花色 {code} 无效，请使用 M、P、S、Z")),
        };
        for digit in digits.drain(..) {
            let rank = digit.to_digit(10).unwrap_or_default() as u8;
            if !(1..=maximum).contains(&rank) {
                return Err(format!("{code} 花色不存在数字 {digit}"));
            }
            let kind = if let Some(suit) = suit {
                MahjongTileKind::suited(suit, rank)
            } else {
                match rank {
                    1 => MahjongTileKind::Wind(MahjongWind::East),
                    2 => MahjongTileKind::Wind(MahjongWind::South),
                    3 => MahjongTileKind::Wind(MahjongWind::West),
                    4 => MahjongTileKind::Wind(MahjongWind::North),
                    5 => MahjongTileKind::Dragon(MahjongDragon::Red),
                    6 => MahjongTileKind::Dragon(MahjongDragon::Green),
                    7 => MahjongTileKind::Dragon(MahjongDragon::White),
                    _ => unreachable!("honor ranks were checked above"),
                }
            };
            tiles.push(kind);
        }
    }
    if !digits.is_empty() {
        return Err("末尾数字缺少花色，请使用 M、P、S 或 Z".to_owned());
    }
    Ok(tiles)
}
