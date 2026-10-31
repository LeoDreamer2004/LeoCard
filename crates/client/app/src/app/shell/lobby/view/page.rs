use super::header::{LobbyPageConfig, add_lobby_rules_heading};
use super::*;
use crate::app::presentation::{MUTED, add_text};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    CozyButtonVariant, LobbyTransitionPanel, LobbyUiAction, PageTransitionElement, UiAction,
    add_configuration_scroll, add_cozy_button_variant, add_cozy_panel,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_protocol::LobbySnapshot;
use leocard_ui::{UiComposer, spawn_node};

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

pub(crate) struct LobbyPage {
    pub rules: Entity,
    pub(crate) players: Entity,
    pub connected_count: u8,
    pub can_configure: bool,
}

impl LobbyPage {
    pub(crate) fn spawn(
        commands: &mut UiComposer,
        root: Entity,
        client: &ClientResource,
        lobby: &LobbySnapshot,
        assets: &UiAssets,
        avatars: &AvatarImages,
        config: LobbyPageConfig,
    ) -> Self {
        let style = config.style;
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let content = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1180),
                height: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                align_self: AlignSelf::Center,
                padding: UiRect::axes(px(22), px(if cfg!(target_os = "android") { 6 } else { 22 })),
                flex_direction: FlexDirection::Row,
                flex_wrap: if style.wrap {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::NoWrap
                },
                row_gap: px(if cfg!(target_os = "android") { 10 } else { 18 }),
                column_gap: px(18),
                align_items: AlignItems::Stretch,
                ..default()
            },
            None,
        );
        let rules = add_cozy_panel(
            commands,
            content,
            Node {
                min_width: px(style.rules_min_width),
                min_height: px(0),
                height: percent(100),
                flex_basis: px(style.rules_basis),
                flex_grow: 1.0,
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(style.rules_gap.min(8.0)),
                ..default()
            },
            assets,
        );
        commands.entity(rules).insert((
            LobbyTransitionPanel(0),
            PageTransitionElement::left(0),
            UiTransform::IDENTITY,
        ));
        let players = add_cozy_panel(
            commands,
            content,
            Node {
                min_width: px(style.players_min_width),
                min_height: px(0),
                height: percent(100),
                flex_basis: px(style.players_basis),
                flex_grow: 2.0,
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(style.players_gap),
                ..default()
            },
            assets,
        );
        commands.entity(players).insert((
            LobbyTransitionPanel(1),
            PageTransitionElement::right(1),
            UiTransform::IDENTITY,
        ));
        let can_configure = client.0.model().you() == lobby.host;
        config.render(commands, root, rules, can_configure, assets);
        let page = Self {
            rules: add_configuration_scroll(
                commands,
                rules,
                format!("lobby-rules:{:?}", lobby.game),
                style.rules_gap.min(8.0),
            ),
            players,
            connected_count: LobbyMetrics::new(lobby).connected_player_count(),
            can_configure,
        };
        page.render_players(commands, client, lobby, assets, avatars);
        page
    }

    fn render_players(
        &self,
        commands: &mut UiComposer,
        client: &ClientResource,
        lobby: &LobbySnapshot,
        assets: &UiAssets,
        avatars: &AvatarImages,
    ) {
        let metrics = LobbyMetrics::new(lobby);
        let capacity = metrics.seat_count();
        let can_start = metrics.can_start();
        add_lobby_rules_heading(
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
            .insert((GlobalZIndex(800), Pickable::IGNORE));
        let you = client.0.model().you();
        let ready = you
            .and_then(|you| lobby.players.iter().find(|player| player.id == you))
            .is_some_and(|player| player.ready);
        add_cozy_button_variant(
            commands,
            actions,
            "退出房间",
            UiAction::Lobby(LobbyUiAction::LeaveRoom),
            assets,
            px(150),
            48.0,
            CozyButtonVariant::Danger,
        );
        if client.0.model().is_spectating() {
            add_disabled_lobby_button(commands, actions, "等待空位", assets);
        } else if you == lobby.host {
            if can_start {
                add_cozy_button_variant(
                    commands,
                    actions,
                    "开始游戏",
                    UiAction::Lobby(LobbyUiAction::StartGame),
                    assets,
                    px(150),
                    48.0,
                    CozyButtonVariant::Neutral,
                );
            } else {
                add_disabled_lobby_button(commands, actions, "等待玩家准备", assets);
            }
        } else {
            add_cozy_button_variant(
                commands,
                actions,
                if ready { "取消准备" } else { "准备" },
                UiAction::Lobby(LobbyUiAction::ToggleReady),
                assets,
                px(150),
                48.0,
                if ready {
                    CozyButtonVariant::Neutral
                } else {
                    CozyButtonVariant::Cool
                },
            );
        }
    }
}

fn add_disabled_lobby_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
) {
    let mut image = ImageNode::new(assets.home.button.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    image.color = Color::WHITE.with_alpha(0.5);
    let button = commands.child(
        parent,
        (
            Node {
                width: px(150),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            image,
            Pickable::IGNORE,
        ),
    );
    add_text(commands, button, label, 14.0, MUTED, assets);
}
