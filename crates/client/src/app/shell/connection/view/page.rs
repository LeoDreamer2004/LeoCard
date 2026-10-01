//! 建房游戏选择与主连接页面。

use super::super::super::{ConnectionUiAction, InputField, PageTransitionElement, UiAction};
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{ButtonHighlight, TEXT, add_avatar, add_text, spawn_node};
use crate::app::runtime::{AppearancePreferences, AvatarImages, ConnectionDraft, UiAssets};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_client::TcpGameClient;
use leocard_protocol::PlayerGender;

use super::controls::*;
use super::input::*;

pub(super) const HOME_PURPLE: Color = Color::srgb(0.78, 0.74, 1.0);
pub(super) const HOME_SOFT: Color = Color::srgb(0.73, 0.79, 0.75);
pub(crate) struct ConnectionScreen<'a> {
    connection: &'a ConnectionDraft,
    appearance: &'a AppearancePreferences,
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> ConnectionScreen<'a> {
    pub(crate) fn new(
        connection: &'a ConnectionDraft,
        appearance: &'a AppearancePreferences,
        network: Option<&'a TcpGameClient>,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            connection,
            appearance,
            network,
            assets,
            avatars,
        }
    }

    pub(crate) fn render(self, commands: &mut Commands, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
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
                max_width: px(1370),
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(16),
                ..default()
            },
            None,
        );
        self.render_games(commands, content);
        self.render_sidebar(commands, content);
    }

    fn render_games(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.1,
                padding: UiRect::all(px(26)),
                flex_direction: FlexDirection::Column,
                row_gap: px(0),
                ..default()
            },
            self.assets,
        );
        commands
            .entity(panel)
            .insert((PageTransitionElement::left(0), UiTransform::IDENTITY));
        let heading = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_text(commands, heading, "马上开局", 30.0, TEXT, self.assets);
        let port = spawn_node(
            commands,
            heading,
            Node {
                width: px(190),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(commands, port, "建房端口", 13.0, HOME_SOFT, self.assets);
        let port_input = spawn_node(
            commands,
            port,
            Node {
                width: px(110),
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            "",
            &self.connection.host_port,
            InputField::HostPort,
            self.connection.active == InputField::HostPort,
            self.connection.selected_all && self.connection.active == InputField::HostPort,
            self.assets,
        )
        .render(commands, port_input);
        spawn_node(
            commands,
            panel,
            Node {
                width: px(94),
                height: px(2),
                ..default()
            },
            Some(HOME_PURPLE.with_alpha(0.75)),
        );

        let shelf = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                margin: UiRect::top(px(16)),
                ..default()
            },
            None,
        );
        for indices in [&[4, 2, 1][..], &[3, 0][..]] {
            let row = spawn_node(
                commands,
                shelf,
                Node {
                    width: percent(100),
                    min_height: px(192),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            for &index in indices {
                self.render_game_card(commands, row, index);
            }
        }
    }

    fn render_game_card(&self, commands: &mut Commands, parent: Entity, index: usize) {
        let game = &SUPPORTED_GAMES[index];
        let card = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::CreateRoom(game.kind)),
                Node {
                    min_width: px(0),
                    min_height: px(0),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    padding: UiRect::axes(px(17), px(14)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    row_gap: px(4),
                    ..default()
                },
                home_game_card_image(self.assets.home.game_card.clone()),
            ))
            .id();
        commands.entity(parent).add_child(card);
        let title_row = spawn_node(
            commands,
            card,
            Node {
                width: percent(100),
                min_height: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                ..default()
            },
            None,
        );
        let left_arrow = add_home_animated_arrow(
            commands,
            title_row,
            card,
            self.assets,
            true,
            Vec2::new(56.0, 35.0),
        );
        add_text(commands, title_row, game.title, 21.0, TEXT, self.assets);
        let right_arrow = add_home_animated_arrow(
            commands,
            title_row,
            card,
            self.assets,
            false,
            Vec2::new(56.0, 35.0),
        );
        let art_holder = spawn_node(
            commands,
            card,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let art_frame = spawn_node(
            commands,
            art_holder,
            Node {
                width: px(72),
                height: px(94),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            (index == 4).then_some(Color::srgb(0.91, 0.90, 0.82)),
        );
        let art = commands
            .spawn((
                Node {
                    width: px(if index == 4 { 58 } else { 68 }),
                    height: px(if index == 4 { 78 } else { 91 }),
                    ..default()
                },
                ImageNode::new(self.assets.home.game_art[index].clone()),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(art_frame).add_child(art);
        add_text(
            commands,
            card,
            game.description,
            15.0,
            HOME_SOFT,
            self.assets,
        );
        commands.entity(card).insert(ButtonHighlight::Card {
            arrows: [left_arrow, right_arrow],
        });
    }

    fn render_sidebar(&self, commands: &mut Commands, parent: Entity) {
        let rail = spawn_node(
            commands,
            parent,
            Node {
                width: percent(32),
                min_width: px(300),
                max_width: px(420),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            None,
        );
        self.render_identity(commands, rail);
        self.render_join(commands, rail);
    }

    fn render_identity(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                width: percent(100),
                padding: UiRect::all(px(21)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            self.assets,
        );
        commands
            .entity(panel)
            .insert((PageTransitionElement::right(1), UiTransform::IDENTITY));
        add_text(commands, panel, "玩家身份", 19.0, TEXT, self.assets);
        let row = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::FlexEnd,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        let avatar_button = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::ChooseAvatar),
                Node {
                    width: px(48),
                    height: px(48),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(percent(50)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.19, 0.28, 0.24)),
            ))
            .id();
        commands.entity(row).add_child(avatar_button);
        add_avatar(
            commands,
            avatar_button,
            &self.connection.player_name,
            self.avatars.local.as_ref(),
            48.0,
            self.assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            "",
            &self.connection.player_name,
            InputField::PlayerName,
            self.connection.active == InputField::PlayerName,
            self.connection.selected_all && self.connection.active == InputField::PlayerName,
            self.assets,
        )
        .render(commands, name);
        let (symbol, color) = match self.connection.gender {
            PlayerGender::Male => ("♂", Color::srgb(0.42, 0.70, 1.0)),
            PlayerGender::Female => ("♀", Color::srgb(1.0, 0.58, 0.76)),
        };
        let gender = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::ToggleGender),
                Node {
                    width: px(48),
                    height: px(48),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .id();
        commands.entity(row).add_child(gender);
        let symbol = add_text(commands, gender, symbol, 31.0, color, self.assets);
        commands.entity(symbol).insert(FocusPolicy::Pass);
        if self.appearance.avatar_png.is_some() {
            home_button(
                commands,
                panel,
                "清除头像",
                ConnectionUiAction::ClearAvatar,
                self.assets,
                false,
                px(92),
            );
        }
    }

    fn render_join(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(0),
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            self.assets,
        );
        commands
            .entity(panel)
            .insert((PageTransitionElement::right(2), UiTransform::IDENTITY));
        add_text(commands, panel, "加入房间", 24.0, TEXT, self.assets);
        add_text(
            commands,
            panel,
            "已有朋友开桌？输入房主地址直接入座。",
            13.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionInput::new(
            "房间地址",
            &self.connection.join_address,
            InputField::JoinAddress,
            self.connection.active == InputField::JoinAddress,
            self.connection.selected_all && self.connection.active == InputField::JoinAddress,
            self.assets,
        )
        .render(commands, panel);
        home_button(
            commands,
            panel,
            "加入房间",
            ConnectionUiAction::JoinRoom,
            self.assets,
            true,
            percent(100),
        );
        add_text(
            commands,
            panel,
            "示例  192.168.1.20:52300",
            12.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionStatus::new(self.network, self.assets).render(commands, panel);
    }
}
