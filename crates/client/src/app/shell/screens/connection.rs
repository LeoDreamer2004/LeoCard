//! 建房游戏选择与主连接页面。

use super::*;
use leocard_client::{NetworkState, TcpGameClient};
use leocard_protocol::GameKind;

const HOST_GAME_CHOICES: [(&str, &str, GameKind); 5] = [
    ("七鬼五二三", "放空大脑, 有牌就出", GameKind::QiGui523),
    ("德州扑克", "窝要验牌!", GameKind::TexasHoldem),
    ("升级", "神对手 or 猪队友", GameKind::Shengji),
    ("UNO", "最后一张，记得喊 UNO!", GameKind::Uno),
    ("麻将合集", "八番起和，方城之战", GameKind::Mahjong),
];

pub(super) struct HostGamePicker<'a> {
    assets: &'a UiAssets,
}

impl<'a> HostGamePicker<'a> {
    pub(super) fn new(assets: &'a UiAssets) -> Self {
        Self { assets }
    }

    pub(super) fn render(self, commands: &mut Commands, root: Entity) {
        let assets = self.assets;
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
            self.add_game_choice(commands, choices, title, description, Some(game));
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
            UiAction::Connection(ConnectionUiAction::CloseHostGamePicker),
            ButtonKind::Secondary,
            assets,
        );
    }

    fn add_game_choice(
        &self,
        commands: &mut Commands,
        parent: Entity,
        title: &str,
        description: &str,
        game: Option<GameKind>,
    ) {
        let assets = self.assets;
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
                UiAction::Connection(ConnectionUiAction::CreateRoom(game)),
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, card, "尚未接入", assets);
        }
    }
}

pub(super) struct ConnectionScreen<'a> {
    form: &'a ConnectionForm,
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> ConnectionScreen<'a> {
    pub(super) fn new(
        form: &'a ConnectionForm,
        network: Option<&'a TcpGameClient>,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            form,
            network,
            assets,
            avatars,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, root: Entity) {
        let Self {
            form,
            network,
            assets,
            avatars,
        } = self;
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

        ConnectionIdentity::new(form, assets, avatars).render(commands, content);
        ConnectionChoices::new(form, assets).render(commands, content);
        ConnectionStatus::new(network, assets).render(commands, content);
    }
}

struct ConnectionIdentity<'a> {
    form: &'a ConnectionForm,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> ConnectionIdentity<'a> {
    fn new(form: &'a ConnectionForm, assets: &'a UiAssets, avatars: &'a AvatarImages) -> Self {
        Self {
            form,
            assets,
            avatars,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let Self {
            form,
            assets,
            avatars,
        } = self;

        let profile_row = spawn_node(
            commands,
            parent,
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
        ConnectionInput::new(
            "玩家名称",
            &form.player_name,
            InputField::PlayerName,
            form.active == InputField::PlayerName,
            assets,
        )
        .render(commands, name_field);
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
                UiAction::Connection(ConnectionUiAction::ChooseAvatar),
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
                UiAction::Connection(ConnectionUiAction::ClearAvatar),
                ButtonKind::Secondary,
                assets,
            );
        }
    }
}

struct ConnectionChoices<'a> {
    form: &'a ConnectionForm,
    assets: &'a UiAssets,
}

impl<'a> ConnectionChoices<'a> {
    fn new(form: &'a ConnectionForm, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let Self { form, assets } = self;
        let choices = spawn_node(
            commands,
            parent,
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
        ConnectionChoicePanel::host(form, assets).render(commands, choices);
        ConnectionChoicePanel::join(form, assets).render(commands, choices);
    }
}

struct ConnectionChoicePanel<'a> {
    title: &'static str,
    description: &'static str,
    background: Color,
    input: ConnectionInput<'a>,
    button_label: &'static str,
    action: UiAction,
    kind: ButtonKind,
    assets: &'a UiAssets,
}

impl<'a> ConnectionChoicePanel<'a> {
    fn host(form: &'a ConnectionForm, assets: &'a UiAssets) -> Self {
        Self {
            title: "开设房间",
            description: "本机将监听所有局域网网卡",
            background: PANEL,
            input: ConnectionInput::new(
                "监听端口",
                &form.host_port,
                InputField::HostPort,
                form.active == InputField::HostPort,
                assets,
            ),
            button_label: "选择游戏并创建",
            action: UiAction::Connection(ConnectionUiAction::OpenHostGamePicker),
            kind: ButtonKind::Primary,
            assets,
        }
    }

    fn join(form: &'a ConnectionForm, assets: &'a UiAssets) -> Self {
        Self {
            title: "加入房间",
            description: "输入房主的局域网地址，例如 192.168.1.20:52300。",
            background: PANEL_ALT,
            input: ConnectionInput::new(
                "连接地址",
                &form.join_address,
                InputField::JoinAddress,
                form.active == InputField::JoinAddress,
                assets,
            ),
            button_label: "连接并加入",
            action: UiAction::Connection(ConnectionUiAction::JoinRoom),
            kind: ButtonKind::Warning,
            assets,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let panel = add_panel(
            commands,
            parent,
            Node {
                min_width: px(360),
                flex_basis: px(470),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            self.background,
            PanelSkin::Section,
            self.assets,
        );
        add_section_title(commands, panel, self.title, self.assets);
        add_text(commands, panel, self.description, 13.0, MUTED, self.assets);
        self.input.render(commands, panel);
        add_action_button(
            commands,
            panel,
            self.button_label,
            self.action,
            self.kind,
            self.assets,
        );
    }
}

struct ConnectionStatus<'a> {
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
}

impl<'a> ConnectionStatus<'a> {
    fn new(network: Option<&'a TcpGameClient>, assets: &'a UiAssets) -> Self {
        Self { network, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        if let Some((status, color)) = self.network.and_then(|network| match network.state() {
            NetworkState::Connecting(message) | NetworkState::Reconnecting(message) => {
                Some((message.as_str(), ACCENT))
            }
            NetworkState::Failed(message) => Some((message.as_str(), DANGER)),
            NetworkState::Connected(_) => None,
        }) {
            add_text(commands, parent, status, 15.0, color, self.assets);
        }
    }
}

struct ConnectionInput<'a> {
    label: &'a str,
    value: &'a str,
    field: InputField,
    active: bool,
    assets: &'a UiAssets,
}

impl<'a> ConnectionInput<'a> {
    fn new(
        label: &'a str,
        value: &'a str,
        field: InputField,
        active: bool,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            label,
            value,
            field,
            active,
            assets,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        add_text(commands, parent, self.label, 13.0, MUTED, self.assets);
        let input = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::FocusInput(self.field)),
                Node {
                    width: percent(100),
                    min_height: px(45),
                    padding: UiRect::axes(px(13), px(9)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(if self.active { 2 } else { 1 })),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(HEADER_BG),
                BorderColor::all(if self.active { ACCENT } else { BORDER }),
            ))
            .id();
        commands.entity(parent).add_child(input);
        add_text(
            commands,
            input,
            format!("{}{}", self.value, if self.active { "│" } else { "" }),
            16.0,
            if self.value.is_empty() { MUTED } else { TEXT },
            self.assets,
        );
    }
}
