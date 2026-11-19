use super::super::page::ConnectionScreen;
use super::editor;
use crate::app::presentation::{PAGE_FRAME_TITLE, add_title, page_frame_skin};
use crate::app::shell::connection::rooms::{JoinRoomDialogKind, JoinRoomUiState};
use crate::app::shell::{
    ConnectionUiAction, CozyModalKind, CozyModalPanel, ShellOverlayLayer, UiAction,
    add_centered_modal, add_cozy_close_button, cozy_panel_transform,
};
use bevy::prelude::*;
use leocard_ui::{PageFrame, UiComposer};

impl ConnectionScreen<'_> {
    pub(crate) fn render_room_dialog(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        state: &JoinRoomUiState,
    ) {
        let Some(dialog) = state.dialog.as_ref() else {
            return;
        };
        if !state.dialog_open && state.dialog_progress == 0.0 {
            return;
        }
        let close = UiAction::Connection(ConnectionUiAction::CancelRoomEdit);
        let overlay = add_centered_modal(
            commands,
            root,
            ShellOverlayLayer::Dialog,
            CozyModalKind::JoinRoomEditor,
            state.dialog_progress,
            state.dialog_open.then(|| close.clone()),
        );
        let mut frame = PageFrame::new(page_frame_skin(self.assets));
        frame.node.width = px(560);
        frame.node.max_width = percent(92);
        frame.node.max_height = percent(92);
        frame.heading.height = px(48);
        frame.content.flex_grow = 0.0;
        frame.content.padding = UiRect::all(px(16));
        frame.content.row_gap = px(16);
        let frame = frame.mount(commands, overlay);
        commands.entity(frame.panel).insert((
            CozyModalPanel(CozyModalKind::JoinRoomEditor),
            cozy_panel_transform(state.dialog_progress),
        ));
        let title = match dialog.kind {
            JoinRoomDialogKind::Add => "添加房间",
            JoinRoomDialogKind::Edit(_) => "编辑房间",
            JoinRoomDialogKind::Direct => "直接连接",
            JoinRoomDialogKind::Delete(_) => "删除房间",
        };
        add_title(
            commands,
            frame.heading,
            title,
            24.0,
            PAGE_FRAME_TITLE,
            self.assets,
        );
        add_cozy_close_button(commands, frame.heading, close, self.assets);
        editor::render(
            commands,
            frame.content,
            dialog,
            state.connecting,
            self.assets,
        );
    }
}
