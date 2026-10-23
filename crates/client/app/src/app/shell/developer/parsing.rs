use leocard_mahjong::{MahjongDragon, MahjongSuit, MahjongTileKind, MahjongWind};
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};
use std::collections::HashMap;

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
