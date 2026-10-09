//! Player identity, room creation settings and room joining beside the gallery.

use super::{
    controls::{home_button, home_panel},
    input::ConnectionInput,
    page::ConnectionScreen,
    theme::HOME_SOFT,
};
use crate::app::presentation::{TEXT, add_avatar, add_text, spawn_node};
use crate::app::shell::{ConnectionUiAction, InputField, PageTransitionElement, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerGender;

impl ConnectionScreen<'_> {
    pub(super) fn render_sidebar(&self, commands: &mut Commands, parent: Entity) {
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
        self.render_create(commands, rail);
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
                width: px(170),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            &self.connection.player_name,
            InputField::PlayerName,
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
                    width: px(32),
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
        commands.entity(symbol).insert(Pickable::IGNORE);
        if self.appearance.avatar_png.is_some() {
            let clear = spawn_node(
                commands,
                row,
                Node {
                    min_width: px(80),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            home_button(
                commands,
                clear,
                "清空头像",
                ConnectionUiAction::ClearAvatar,
                self.assets,
                false,
                percent(100),
            );
        }
    }

    fn room_panel(
        &self,
        commands: &mut Commands,
        parent: Entity,
        title: &str,
        order: u8,
    ) -> Entity {
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
            .insert((PageTransitionElement::right(order), UiTransform::IDENTITY));
        add_text(commands, panel, title, 24.0, TEXT, self.assets);
        panel
    }

    fn render_create(&self, commands: &mut Commands, parent: Entity) {
        let panel = self.room_panel(commands, parent, "创建房间", 2);
        let row = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_text(commands, row, "创建端口", 12.0, HOME_SOFT, self.assets);
        let input = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            &self.connection.host_port,
            InputField::HostPort,
            self.assets,
        )
        .render(commands, input);
    }

    fn render_join(&self, commands: &mut Commands, parent: Entity) {
        let panel = self.room_panel(commands, parent, "加入房间", 3);
        let heading = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(commands, heading, "房间地址", 12.0, HOME_SOFT, self.assets);
        add_text(
            commands,
            heading,
            "示例：192.168.1.20:52300",
            11.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionInput::new(
            &self.connection.join_address,
            InputField::JoinAddress,
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
    }
}
