//! 首页入房设置与离线页面共用的个人信息编辑浮层。

use super::super::lan::{LanDiscovery, LanUiState};
use super::{
    controls::{home_button, home_panel},
    input::ConnectionInput,
    overlay::HomeIdentityPanel,
    page::ConnectionScreen,
};
use crate::app::presentation::{add_avatar, add_text};
use crate::app::shell::{
    ConnectionUiAction, InputField, ModalDismiss, ModalInputBlocker, PageTransitionElement,
    UiAction,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerGender;
use leocard_ui::{UiComposer, spawn_node};

impl ConnectionScreen<'_> {
    pub(crate) fn render_home_overlays(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lan: &LanUiState,
        discovery: &LanDiscovery,
    ) {
        let joining = self.home.join_open || self.home.join_progress > 0.0;
        if joining {
            self.render_join_modal(commands, root, lan, discovery);
            return;
        }
        if !self.home.identity_open && self.home.identity_progress == 0.0 {
            return;
        }
        let layer = commands.child(
            root,
            (
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
                GlobalZIndex(1400),
                ModalInputBlocker,
            ),
        );
        let close = UiAction::Connection(ConnectionUiAction::ToggleIdentity);
        if self.home.identity_open {
            commands.entity(layer).insert(ModalDismiss(close.clone()));
        }
        commands.child(
            layer,
            (
                Button,
                close,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
            ),
        );
        let panel = spawn_node(
            commands,
            layer,
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(86),
                width: px(460),
                max_width: percent(94),
                max_height: percent(90),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(panel)
            .insert((HomeIdentityPanel, UiTransform::IDENTITY));
        self.render_identity(commands, panel);
    }

    fn render_identity(&self, commands: &mut UiComposer, parent: Entity) {
        let frame = home_panel(commands, parent, "玩家身份", self.assets);
        commands
            .entity(frame.panel)
            .insert((PageTransitionElement::right(1), UiTransform::IDENTITY));
        let row = spawn_node(
            commands,
            frame.content,
            Node {
                width: percent(100),
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(8),
                align_items: AlignItems::FlexEnd,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        let avatar_button = commands.child(
            row,
            (
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
            ),
        );
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
                min_width: px(100),
                flex_basis: px(0),
                flex_grow: 1.0,
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
        let gender = commands.child(
            row,
            (
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
            ),
        );
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
}
