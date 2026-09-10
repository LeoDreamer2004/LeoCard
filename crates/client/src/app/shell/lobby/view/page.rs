use super::*;
use crate::app::presentation::{
    ButtonKind, PANEL, PANEL_ALT, PanelSkin, add_action_button, add_disabled_action_button,
    add_panel, add_section_title, spawn_node,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::LobbySnapshot;

#[derive(Clone, Copy)]
pub(crate) struct LobbyPageStyle {
    pub rules_min_width: f32,
    pub rules_basis: f32,
    pub rules_gap: f32,
    pub players_min_width: f32,
    pub players_basis: f32,
    pub players_gap: f32,
    pub wrap: bool,
}

impl Default for LobbyPageStyle {
    fn default() -> Self {
        Self {
            rules_min_width: 300.0,
            rules_basis: 330.0,
            rules_gap: 14.0,
            players_min_width: 380.0,
            players_basis: 560.0,
            players_gap: 10.0,
            wrap: true,
        }
    }
}

pub(crate) struct LobbyPlayerSection<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
    capacity: u8,
    can_start: bool,
    waiting_label: &'a str,
}

impl<'a> LobbyPlayerSection<'a> {
    pub(crate) fn new(
        client: &'a ClientResource,
        lobby: &'a LobbySnapshot,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
        capacity: u8,
        can_start: bool,
        waiting_label: &'a str,
    ) -> Self {
        Self {
            client,
            lobby,
            assets,
            avatars,
            capacity,
            can_start,
            waiting_label,
        }
    }
}

pub(crate) struct LobbyPage {
    pub rules: Entity,
    players: Entity,
    pub connected_count: usize,
    pub can_configure: bool,
}

impl LobbyPage {
    pub(crate) fn spawn(
        commands: &mut Commands,
        root: Entity,
        client: &ClientResource,
        lobby: &LobbySnapshot,
        assets: &UiAssets,
        style: LobbyPageStyle,
    ) -> Self {
        let content = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                max_width: px(1180),
                flex_grow: 1.0,
                align_self: AlignSelf::Center,
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Row,
                flex_wrap: if style.wrap {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::NoWrap
                },
                row_gap: px(18),
                column_gap: px(18),
                align_items: AlignItems::Stretch,
                ..default()
            },
            None,
        );
        let rules = add_panel(
            commands,
            content,
            Node {
                min_width: px(style.rules_min_width),
                flex_basis: px(style.rules_basis),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(style.rules_gap),
                ..default()
            },
            PANEL,
            PanelSkin::Section,
            assets,
        );
        let players = add_panel(
            commands,
            content,
            Node {
                min_width: px(style.players_min_width),
                flex_basis: px(style.players_basis),
                flex_grow: 2.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(style.players_gap),
                ..default()
            },
            PANEL_ALT,
            PanelSkin::Section,
            assets,
        );
        Self {
            rules,
            players,
            connected_count: LobbyMetrics::new(lobby).connected_player_count(),
            can_configure: client.0.model().you() == lobby.host,
        }
    }

    pub(crate) fn render_players(self, commands: &mut Commands, section: LobbyPlayerSection<'_>) {
        let LobbyPlayerSection {
            client,
            lobby,
            assets,
            avatars,
            capacity,
            can_start,
            waiting_label,
        } = section;
        add_section_title(
            commands,
            self.players,
            format!("玩家席位  {}/{}", self.connected_count, capacity),
            assets,
        );
        LobbySeatSelector::new(client, lobby, assets, avatars).render(commands, self.players);
        let actions = spawn_node(
            commands,
            self.players,
            Node {
                width: percent(100),
                min_height: px(48),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            None,
        );
        commands
            .entity(actions)
            .insert((GlobalZIndex(800), FocusPolicy::Pass));
        let you = client.0.model().you();
        let ready = you
            .and_then(|you| lobby.players.iter().find(|player| player.id == you))
            .is_some_and(|player| player.ready);
        add_action_button(
            commands,
            actions,
            "退出房间",
            UiAction::Lobby(LobbyUiAction::LeaveRoom),
            ButtonKind::Pass,
            assets,
        );
        if you == lobby.host {
            if can_start {
                add_action_button(
                    commands,
                    actions,
                    "开始游戏",
                    UiAction::Lobby(LobbyUiAction::StartGame),
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(commands, actions, waiting_label, assets);
            }
        } else {
            add_action_button(
                commands,
                actions,
                if ready { "取消准备" } else { "准备" },
                UiAction::Lobby(LobbyUiAction::ToggleReady),
                if ready {
                    ButtonKind::Secondary
                } else {
                    ButtonKind::Primary
                },
                assets,
            );
        }
    }
}
