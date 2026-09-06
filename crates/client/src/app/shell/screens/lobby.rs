//! 通用准备大厅分派与席位组件。

use super::*;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::{GameKind, SeatId, TABLE_SEAT_COUNT};
use leocard_protocol::{GameRules, LobbySnapshot};
use leocard_shengji::ShengjiRuleSet;
use leocard_uno::UnoRuleSet;

pub fn render_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    ui: &UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    match lobby.game {
        GameKind::QiGui523 => render_qigui523_lobby(commands, root, client, lobby, assets, avatars),
        GameKind::TexasHoldem => {
            render_texas_holdem_lobby(commands, root, client, lobby, assets, avatars)
        }
        GameKind::Shengji => render_shengji_lobby(commands, root, client, lobby, assets, avatars),
        GameKind::Uno => render_uno_lobby(commands, root, client, lobby, ui, assets, avatars),
        GameKind::Mahjong => render_mahjong_lobby(commands, root, client, lobby, assets, avatars),
    }
}

pub fn render_seat_selector(
    commands: &mut Commands,
    parent: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let ring = spawn_node(
        commands,
        parent,
        Node {
            width: px(620),
            height: px(390),
            max_width: percent(100),
            align_self: AlignSelf::Center,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let table = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(150),
                top: px(108),
                width: px(320),
                height: px(174),
                padding: UiRect::axes(px(24), px(18)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                overflow: Overflow::clip(),
                ..default()
            },
            ImageNode::new(assets.table_felt.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgba(0.72, 0.83, 0.76, 0.92)),
            BackgroundColor(TABLE_BG),
            BorderColor::all(Color::srgba(0.62, 0.82, 0.70, 0.28)),
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(7), px(0), px(9)),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(ring).add_child(table);
    let connected_count = connected_lobby_player_count(lobby);
    let ready_count = lobby
        .players
        .iter()
        .filter(|player| player.connected && player.ready)
        .count();
    add_text(
        commands,
        table,
        format!("等待准备  {ready_count}/{connected_count}"),
        17.0,
        TEXT,
        assets,
    );
    #[cfg(feature = "developer")]
    if client.0.model().you() == lobby.host {
        add_text(
            commands,
            table,
            "右键空座添加机器人，右键机器人移除",
            10.5,
            MUTED,
            assets,
        );
    }
    let rule_chips = spawn_node(
        commands,
        table,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let rule_labels = match &lobby.rules {
        GameRules::QiGui523(rules) => vec![
            format!("{}副", rules.deck_count),
            format!("{}张", rules.hand_size),
            time_control_label(rules.time_control).to_owned(),
        ],
        GameRules::TexasHoldem(rules) => vec![
            format!("{}筹码", rules.starting_chips),
            match (rules.omaha, rules.short_deck) {
                (true, true) => "短牌奥马哈".to_owned(),
                (true, false) => "奥马哈".to_owned(),
                (false, true) => "短牌德州".to_owned(),
                (false, false) => "标准德州".to_owned(),
            },
            if rules.ignore_kickers {
                "只比较最大牌型".to_owned()
            } else {
                "标准比牌".to_owned()
            },
        ],
        GameRules::Shengji(rules) => vec![
            format!("{}副牌", rules.deck_count),
            if rules.bottom_copy {
                "允许抄底".to_owned()
            } else {
                "不抄底".to_owned()
            },
            if rules.five_trump_crossing {
                "五主过江".to_owned()
            } else {
                "不过江".to_owned()
            },
        ],
        GameRules::Uno(rules) if rules.is_no_mercy() => vec![
            "No Mercy".to_owned(),
            "+2 至 +10 递增堆叠".to_owned(),
            if rules.no_mercy.mercy_elimination {
                "25 张淘汰".to_owned()
            } else {
                "不启用慈悲淘汰".to_owned()
            },
        ],
        GameRules::Uno(rules) if rules.is_flip() => vec![
            "UNO FLIP".to_owned(),
            if rules.flip.random_pairing {
                "随机双面配对".to_owned()
            } else {
                "固定双面配对".to_owned()
            },
            if rules.flip.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
        ],
        GameRules::Uno(rules) => vec![
            if rules.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
            if rules.jump_in {
                "允许抢出".to_owned()
            } else {
                "不抢出".to_owned()
            },
            if rules.uno_callout {
                "UNO 检举".to_owned()
            } else {
                "不检举".to_owned()
            },
        ],
        GameRules::Mahjong(rules) => vec![
            match rules.match_length {
                MahjongMatchLength::SingleHand => "单局结算".to_owned(),
                MahjongMatchLength::EastRound => "东风场".to_owned(),
                MahjongMatchLength::HalfGame => "半庄场".to_owned(),
                MahjongMatchLength::FullGame => "全庄场".to_owned(),
            },
            if rules.minimum_eight_points {
                "8 番起和".to_owned()
            } else {
                "不限起和番数".to_owned()
            },
            if rules.multiple_winners {
                "允许一炮多响".to_owned()
            } else {
                "截和".to_owned()
            },
        ],
    };
    for label in rule_labels {
        add_lobby_rule_chip(commands, rule_chips, label, assets);
    }

    let you = client.0.model().you();
    let seat_count = match &lobby.rules {
        GameRules::Shengji(_) => ShengjiRuleSet::PLAYER_COUNT as u8,
        GameRules::Uno(_) => UnoRuleSet::MAX_PLAYERS,
        GameRules::Mahjong(_) => MahjongRuleSet::PLAYER_COUNT as u8,
        GameRules::QiGui523(_) | GameRules::TexasHoldem(_) => TABLE_SEAT_COUNT,
    };
    for seat_index in 0..seat_count {
        let seat = SeatId(seat_index);
        let occupant = lobby
            .players
            .iter()
            .find(|player| player.seat == Some(seat));
        let is_you = occupant.is_some_and(|player| Some(player.id) == you);
        let is_host = occupant.is_some_and(|player| Some(player.id) == lobby.host);
        let (left, top) = if matches!(lobby.game, GameKind::Shengji | GameKind::Mahjong) {
            match seat_index {
                0 => (239.0, 290.0),
                1 => (0.0, 145.0),
                2 => (239.0, 0.0),
                3 => (478.0, 145.0),
                _ => unreachable!("双升和麻将固定四个座位"),
            }
        } else {
            lobby_seat_position(seat_index)
        };
        let entity = commands
            .spawn((
                Button,
                UiAction::SelectSeat(seat),
                LobbySeatHover {
                    seat: seat_index,
                    amount: 0.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(142),
                    height: px(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                UiTransform::IDENTITY,
            ))
            .id();
        commands.entity(ring).add_child(entity);
        let visual = spawn_node(
            commands,
            entity,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        commands.entity(visual).insert((
            LobbySeatVisual(seat_index),
            UiTransform::IDENTITY,
            FocusPolicy::Pass,
        ));
        if let Some(player) = occupant {
            commands
                .entity(entity)
                .insert(LobbySeatTransitionSource(player.id));
            let avatar_ring = spawn_node(
                commands,
                visual,
                Node {
                    width: px(60),
                    height: px(60),
                    min_width: px(60),
                    position_type: PositionType::Relative,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(if is_you { 3 } else { 2 })),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::srgba(0.02, 0.08, 0.06, 0.80)),
            );
            commands.entity(avatar_ring).insert((
                BorderColor::all(if is_you {
                    ACCENT
                } else if player.ready {
                    READY
                } else {
                    MUTED.with_alpha(0.42)
                }),
                BoxShadow::new(
                    if is_you {
                        ACCENT.with_alpha(0.24)
                    } else {
                        Color::BLACK.with_alpha(0.24)
                    },
                    px(0),
                    px(2),
                    px(0),
                    px(5),
                ),
            ));
            let handle = player.avatar.and_then(|id| avatars.remote.get(&id));
            let avatar = add_avatar(commands, avatar_ring, &player.name, handle, 52.0, assets);
            if is_host {
                add_host_crown(commands, avatar, assets);
            }
            if player.ready {
                let check = spawn_node(
                    commands,
                    avatar_ring,
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(-2),
                        bottom: px(-1),
                        width: px(18),
                        height: px(18),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    Some(READY),
                );
                add_text(commands, check, "✓", 11.5, Color::WHITE, assets);
            }
            add_text(
                commands,
                visual,
                &player.name,
                14.0,
                if is_you { ACCENT } else { TEXT },
                assets,
            );
            let status = spawn_node(
                commands,
                visual,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(5),
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                status,
                reference_level(player.reference_points),
                10.5,
                MUTED,
                assets,
            );
            add_text(
                commands,
                status,
                if player.ready {
                    "已准备"
                } else {
                    "未准备"
                },
                12.5,
                if player.ready { READY } else { MUTED },
                assets,
            );
        } else {
            let empty_ring = spawn_node(
                commands,
                visual,
                Node {
                    width: px(56),
                    height: px(56),
                    position_type: PositionType::Relative,
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::srgba(0.03, 0.12, 0.085, 0.64)),
            );
            commands.entity(empty_ring).insert((
                LobbyEmptySeatRing(seat_index),
                BorderColor::all(MUTED.with_alpha(0.34)),
            ));
            for node in [
                Node {
                    position_type: PositionType::Absolute,
                    left: px(15),
                    top: px(25),
                    width: px(22),
                    height: px(2),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(25),
                    top: px(15),
                    width: px(2),
                    height: px(22),
                    ..default()
                },
            ] {
                let stroke = spawn_node(commands, empty_ring, node, Some(MUTED.with_alpha(0.72)));
                commands.entity(stroke).insert(FocusPolicy::Pass);
            }
            let label = add_text(commands, visual, "空位", 12.0, MUTED, assets);
            commands
                .entity(label)
                .insert(LobbyEmptySeatLabel(seat_index));
        }
    }
}

pub fn connected_lobby_player_count(lobby: &LobbySnapshot) -> usize {
    lobby
        .players
        .iter()
        .filter(|player| player.connected)
        .count()
}

fn add_lobby_rule_chip(
    commands: &mut Commands,
    parent: Entity,
    label: impl Into<String>,
    assets: &UiAssets,
) {
    let chip = spawn_node(
        commands,
        parent,
        Node {
            min_height: px(25),
            padding: UiRect::axes(px(8), px(3)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(Color::srgba(0.015, 0.065, 0.048, 0.72)),
    );
    commands
        .entity(chip)
        .insert(BorderColor::all(Color::srgba(0.70, 0.88, 0.78, 0.18)));
    add_text(commands, chip, label, 11.5, TEXT, assets);
}
