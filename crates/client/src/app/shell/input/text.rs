//! 文本输入、聊天输入与开发者手牌语法。

use super::*;
#[cfg(feature = "developer")]
use leocard_mahjong::{
    MahjongDragon, MahjongHandReplacementError, MahjongSuit, MahjongTileKind, MahjongWind,
};
use leocard_protocol::{ChatContent, ClientCommand, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS};
#[cfg(feature = "developer")]
use leocard_protocol::{GameCommand, MahjongCommand, QiGui523Command};
#[cfg(feature = "developer")]
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};
#[cfg(feature = "developer")]
use std::collections::HashMap;

pub fn developer_hand_input_label(input: &DeveloperHandInput, placeholder: &'static str) -> String {
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

pub fn sync_developer_hand_input_text(
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

pub fn animate_turn_clocks(
    time: Res<Time>,
    mut clocks: Query<(&mut UiTransform, &mut BorderColor), With<TurnClock>>,
    mut hands: Query<&mut UiTransform, (With<TurnClockHand>, Without<TurnClock>)>,
) {
    let elapsed = time.elapsed_secs();
    let ring = (elapsed * 8.0).sin();
    for (mut transform, mut border) in &mut clocks {
        transform.scale = Vec2::splat(1.04 + ring.abs() * 0.05);
        transform.rotation = Rot2::radians(ring * 0.055);
        border.set_all(ACCENT.with_alpha(0.72 + ring.abs() * 0.28));
    }
    for mut transform in &mut hands {
        transform.rotation = Rot2::radians(elapsed * 2.8);
    }
}

pub fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut ime_inputs: MessageReader<Ime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<Clipboard>,
    mut form: ResMut<ConnectionForm>,
    mut chat: ResMut<ChatPanelState>,
    mut developer_hand: ResMut<DeveloperHandInput>,
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<UiState>,
) {
    for input in ime_inputs.read() {
        let Ime::Commit { value, .. } = input else {
            continue;
        };
        if chat.focused {
            append_chat_input(&mut chat.input, value);
        } else if !developer_hand.focused && form.active == InputField::PlayerName {
            append_filtered_input(
                &mut form.player_name,
                InputField::PlayerName,
                value,
                MAX_PLAYER_NAME_CHARS,
            );
            form.error = None;
            ui.dirty = true;
        }
    }

    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if control {
        if keyboard.just_pressed(KeyCode::KeyV) && developer_hand.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_developer_hand_input(&mut developer_hand.value, &text);
            }
        } else if keyboard.just_pressed(KeyCode::KeyV) && chat.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_chat_input(&mut chat.input, &text);
            }
        } else if form.active == InputField::JoinAddress && keyboard.just_pressed(KeyCode::KeyV) {
            let mut read = clipboard.fetch_text();
            match read.poll_result() {
                Some(Ok(text)) => {
                    let mut address = String::new();
                    append_filtered_input(&mut address, InputField::JoinAddress, &text, 64);
                    if address.is_empty() {
                        form.error = Some("剪贴板中没有可用的服务器地址".to_owned());
                    } else {
                        form.join_address = address;
                        form.error = None;
                    }
                }
                Some(Err(error)) => {
                    form.error = Some(format!("无法读取剪贴板：{error}"));
                }
                None => {
                    form.error = Some("剪贴板内容尚未准备好".to_owned());
                }
            }
            ui.dirty = true;
        }
        return;
    }
    for input in keyboard_inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        match input.logical_key {
            Key::Backspace => {
                if developer_hand.focused {
                    developer_hand.value.pop();
                    form.error = None;
                    continue;
                }
                if chat.focused {
                    chat.input.pop();
                    continue;
                }
                active_input_mut(&mut form).pop();
                form.error = None;
                ui.dirty = true;
            }
            Key::Tab => {
                if chat.focused || developer_hand.focused {
                    continue;
                }
                form.active = match form.active {
                    InputField::PlayerName => InputField::HostPort,
                    InputField::HostPort => InputField::JoinAddress,
                    InputField::JoinAddress => InputField::PlayerName,
                };
                ui.dirty = true;
            }
            Key::Enter if chat.focused => {
                let message = chat.input.trim();
                if !message.is_empty()
                    && let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Chat {
                        content: ChatContent::Text(message.to_owned()),
                    })
                {
                    chat.input.clear();
                }
            }
            Key::Enter if developer_hand.focused => {
                developer_hand.focused = false;
                let input = developer_hand.value.trim();
                if input.is_empty() {
                    form.error = None;
                    continue;
                }
                #[cfg(feature = "developer")]
                {
                    let Some(client) = client.as_deref_mut() else {
                        continue;
                    };
                    let command = if let Some(game) = client.0.model().mahjong_game() {
                        parse_developer_mahjong_hand(input).and_then(|tiles| {
                            if tiles.len() != game.your_hand.len() {
                                return Err(MahjongHandReplacementError::WrongTileCount {
                                    expected: game.your_hand.len() as u16,
                                    actual: tiles.len() as u16,
                                }
                                .to_string());
                            }
                            Ok(ClientCommand::Game(GameCommand::Mahjong(
                                MahjongCommand::SetDeveloperHand { tiles },
                            )))
                        })
                    } else {
                        parse_developer_hand(input).map(|cards| {
                            ClientCommand::Game(GameCommand::QiGui523(
                                QiGui523Command::SetDeveloperHand { cards },
                            ))
                        })
                    };
                    match command {
                        Ok(command) => {
                            if client.0.send(command) {
                                form.error = None;
                                developer_hand.value.clear();
                            } else {
                                form.error = Some("当前未连接，无法编辑开发者手牌".to_owned());
                            }
                        }
                        Err(error) => form.error = Some(error),
                    }
                }
            }
            Key::Escape if chat.focused => {
                chat.focused = false;
                chat.quick_voice_open = false;
            }
            Key::Escape if developer_hand.focused => {
                developer_hand.focused = false;
            }
            _ => {
                let Some(text) = input.text.as_deref() else {
                    continue;
                };
                if chat.focused {
                    append_chat_input(&mut chat.input, text);
                    continue;
                }
                if developer_hand.focused {
                    append_developer_hand_input(&mut developer_hand.value, text);
                    form.error = None;
                    continue;
                }
                let active = form.active;
                let value = active_input_mut(&mut form);
                let maximum = match active {
                    InputField::PlayerName => MAX_PLAYER_NAME_CHARS,
                    InputField::HostPort => 5,
                    InputField::JoinAddress => 64,
                };
                append_filtered_input(value, active, text, maximum);
                form.error = None;
                ui.dirty = true;
            }
        }
    }
}

pub fn append_chat_input(value: &mut String, text: &str) {
    for character in text.chars().filter(|character| !character.is_control()) {
        if value.chars().count() >= MAX_CHAT_MESSAGE_CHARS {
            break;
        }
        value.push(character);
    }
}

fn append_developer_hand_input(value: &mut String, text: &str) {
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

pub fn append_filtered_input(value: &mut String, field: InputField, text: &str, maximum: usize) {
    for character in text.chars().filter(|character| !character.is_control()) {
        let allowed = match field {
            InputField::PlayerName => true,
            InputField::HostPort => character.is_ascii_digit(),
            InputField::JoinAddress => {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | ':' | '[' | ']' | '-')
            }
        };
        if allowed && value.chars().count() < maximum {
            value.push(character);
        }
    }
}

fn active_input_mut(form: &mut ConnectionForm) -> &mut String {
    match form.active {
        InputField::PlayerName => &mut form.player_name,
        InputField::HostPort => &mut form.host_port,
        InputField::JoinAddress => &mut form.join_address,
    }
}
