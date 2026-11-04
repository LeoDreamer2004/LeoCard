use super::controls::home_button;
use super::input::ConnectionInput;
use super::page::ConnectionScreen;
use super::theme::HOME_SOFT;
use crate::app::presentation::{PAGE_FRAME_TITLE, add_text, add_title, page_frame_skin};
use crate::app::shell::{
    ConnectionUiAction, CozyModalBackdrop, CozyModalKind, CozyModalPanel, InputField, UiAction,
    add_cozy_close_button, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{PageFrame, UiComposer, spawn_node};

impl ConnectionScreen<'_> {
    pub(super) fn render_join_modal(&self, commands: &mut UiComposer, root: Entity) {
        let progress = self.home.join_progress;
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
            Some(cozy_backdrop_color(progress)),
        );
        commands.entity(overlay).insert((
            GlobalZIndex(2000),
            CozyModalBackdrop(CozyModalKind::JoinRoom),
        ));
        commands.child(
            overlay,
            (
                Button,
                UiAction::Connection(ConnectionUiAction::ToggleJoin),
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
        let mut frame = PageFrame::new(page_frame_skin(self.assets));
        frame.node.width = px(480);
        frame.node.max_width = percent(92);
        frame.node.max_height = percent(92);
        frame.heading.height = px(48);
        frame.content.flex_grow = 0.0;
        frame.content.padding = UiRect::all(px(16));
        frame.content.row_gap = px(12);
        let frame = frame.mount(commands, overlay);
        commands.entity(frame.panel).insert((
            CozyModalPanel(CozyModalKind::JoinRoom),
            cozy_panel_transform(progress),
        ));
        add_title(
            commands,
            frame.heading,
            "加入房间",
            26.0,
            PAGE_FRAME_TITLE,
            self.assets,
        );
        add_cozy_close_button(
            commands,
            frame.heading,
            UiAction::Connection(ConnectionUiAction::ToggleJoin),
            self.assets,
        );
        let panel = frame.content;
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
        add_text(commands, heading, "房间地址", 14.0, HOME_SOFT, self.assets);
        add_text(
            commands,
            heading,
            "示例：192.168.1.20:52300",
            12.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionInput::new(
            &self.connection.join_address,
            InputField::JoinAddress,
            self.assets,
        )
        .render(commands, panel);
        let row = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                column_gap: px(10),
                ..default()
            },
            None,
        );
        for (label, action, primary) in [
            ("局域网联机", ConnectionUiAction::OpenLan, false),
            ("加入房间", ConnectionUiAction::JoinRoom, true),
        ] {
            let slot = spawn_node(
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
            home_button(
                commands,
                slot,
                label,
                action,
                self.assets,
                primary,
                percent(100),
            );
        }
    }
}
