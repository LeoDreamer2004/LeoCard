//! 建房游戏选择与主连接页面。

use super::*;

const HOST_GAME_CHOICES: [(&str, &str, GameKind); 5] = [
    ("七鬼五二三", "放空大脑, 有牌就出", GameKind::QiGui523),
    ("德州扑克", "窝要验牌!", GameKind::TexasHoldem),
    ("升级", "神对手 or 猪队友", GameKind::Shengji),
    ("UNO", "最后一张，记得喊 UNO!", GameKind::Uno),
    ("麻将合集", "八番起和，方城之战", GameKind::Mahjong),
];

pub fn render_host_game_picker(commands: &mut Commands, root: Entity, assets: &UiAssets) {
    let overlay = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::srgba(0.005, 0.015, 0.012, 0.78)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2100), FocusPolicy::Block));

    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(1050),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "选择游戏", assets);
    add_text(
        commands,
        modal,
        "选择本房间要进行的棋牌游戏。进入等待大厅后可继续配置该游戏的规则。",
        14.0,
        MUTED,
        assets,
    );

    let choices = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            column_gap: px(16),
            ..default()
        },
        None,
    );
    for (title, description, game) in HOST_GAME_CHOICES {
        add_host_game_choice(commands, choices, title, description, Some(game), assets);
    }

    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "取消",
        UiAction::CloseHostGamePicker,
        ButtonKind::Secondary,
        assets,
    );
}

fn add_host_game_choice(
    commands: &mut Commands,
    parent: Entity,
    title: &str,
    description: &str,
    game: Option<GameKind>,
    assets: &UiAssets,
) {
    let card = add_panel(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(190),
            flex_basis: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: px(12),
            ..default()
        },
        if game.is_some() {
            PANEL_ALT
        } else {
            HEADER_BG.with_alpha(0.72)
        },
        PanelSkin::Section,
        assets,
    );
    let copy = spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        copy,
        title,
        24.0,
        if game.is_some() { ACCENT } else { MUTED },
        assets,
    );
    add_text(commands, copy, description, 14.0, MUTED, assets);
    if let Some(game) = game {
        add_action_button(
            commands,
            card,
            "创建房间",
            UiAction::CreateRoom(game),
            ButtonKind::Primary,
            assets,
        );
    } else {
        add_disabled_action_button(commands, card, "尚未接入", assets);
    }
}

pub fn render_connection(
    commands: &mut Commands,
    root: Entity,
    form: &ConnectionForm,
    network: Option<&TcpGameClient>,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1050),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(28)),
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );

    let profile_row = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(28),
            ..default()
        },
        None,
    );
    let name_field = spawn_node(
        commands,
        profile_row,
        Node {
            width: px(260),
            max_width: px(260),
            min_width: px(220),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        None,
    );
    add_input(
        commands,
        name_field,
        "玩家名称",
        &form.player_name,
        InputField::PlayerName,
        form.active == InputField::PlayerName,
        assets,
    );
    let avatar_row = spawn_node(
        commands,
        profile_row,
        Node {
            width: px(365),
            min_width: px(300),
            flex_shrink: 0.0,
            min_height: px(58),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let avatar_button = commands
        .spawn((
            Button,
            UiAction::ChooseAvatar,
            Node {
                width: px(54),
                height: px(54),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(PANEL_ALT),
            BorderColor::all(ACCENT.with_alpha(0.7)),
        ))
        .id();
    commands.entity(avatar_row).add_child(avatar_button);
    add_avatar(
        commands,
        avatar_button,
        &form.player_name,
        avatars.local.as_ref(),
        48.0,
        assets,
    );
    let avatar_help = spawn_node(
        commands,
        avatar_row,
        Node {
            min_width: px(0),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    add_text(commands, avatar_help, "个人头像", 14.0, TEXT, assets);
    add_text(
        commands,
        avatar_help,
        "点击头像更换图片",
        12.0,
        MUTED,
        assets,
    );
    if form.avatar_png.is_some() {
        add_action_button(
            commands,
            avatar_row,
            "清除头像",
            UiAction::ClearAvatar,
            ButtonKind::Secondary,
            assets,
        );
    }

    let choices = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(18),
            row_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let host = add_panel(
        commands,
        choices,
        Node {
            min_width: px(360),
            flex_basis: px(470),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, host, "开设房间", assets);
    add_text(
        commands,
        host,
        "本机将监听所有局域网网卡",
        13.0,
        MUTED,
        assets,
    );
    add_input(
        commands,
        host,
        "监听端口",
        &form.host_port,
        InputField::HostPort,
        form.active == InputField::HostPort,
        assets,
    );
    add_action_button(
        commands,
        host,
        "选择游戏并创建",
        UiAction::OpenHostGamePicker,
        ButtonKind::Primary,
        assets,
    );

    let join = add_panel(
        commands,
        choices,
        Node {
            min_width: px(360),
            flex_basis: px(470),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, join, "加入房间", assets);
    add_text(
        commands,
        join,
        "输入房主的局域网地址，例如 192.168.1.20:52300。",
        13.0,
        MUTED,
        assets,
    );
    add_input(
        commands,
        join,
        "连接地址",
        &form.join_address,
        InputField::JoinAddress,
        form.active == InputField::JoinAddress,
        assets,
    );
    add_action_button(
        commands,
        join,
        "连接并加入",
        UiAction::JoinRoom,
        ButtonKind::Warning,
        assets,
    );

    if let Some((status, color)) = network.and_then(|network| match network.state() {
        NetworkState::Connecting(message) | NetworkState::Reconnecting(message) => {
            Some((message.as_str(), ACCENT))
        }
        NetworkState::Failed(message) => Some((message.as_str(), DANGER)),
        NetworkState::Connected(_) => None,
    }) {
        add_text(commands, content, status, 15.0, color, assets);
    }
}

fn add_input(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    field: InputField,
    active: bool,
    assets: &UiAssets,
) {
    add_text(commands, parent, label, 13.0, MUTED, assets);
    let input = commands
        .spawn((
            Button,
            UiAction::FocusInput(field),
            Node {
                width: percent(100),
                min_height: px(45),
                padding: UiRect::axes(px(13), px(9)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(if active { 2 } else { 1 })),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(if active { ACCENT } else { BORDER }),
        ))
        .id();
    commands.entity(parent).add_child(input);
    add_text(
        commands,
        input,
        format!("{value}{}", if active { "│" } else { "" }),
        16.0,
        if value.is_empty() { MUTED } else { TEXT },
        assets,
    );
}
