use super::MahjongLinkUiAction;
use super::layout;
use super::playback::LinkPlayback;
use super::state::{MahjongLinkUiState, can_act};
use crate::app::presentation::mahjong::MahjongProjectionViewport;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    ConfirmationDialog, PageMotion, PressedUiAction, UiAction, UiState, game_command,
};
use bevy::ecs::system::SystemParam;
#[cfg(target_os = "android")]
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::events::{
    PointerCancel, PointerClick, PointerDrag, PointerDragEnd, PointerDragStart,
};
use bevy::picking::pointer::{PointerButton, PointerId};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_client::NetworkState;
use leocard_mahjong_link::{
    BOARD_COLUMNS, LinkCell, LinkMovement, MahjongLinkAction, MahjongLinkBoard, PushSide,
};
use leocard_protocol::{GameSnapshot, MahjongLinkCommand};

#[derive(Component)]
pub(super) struct LinkTileHitbox(pub LinkCell);

#[derive(Clone, Copy)]
pub(super) struct LinkPushRange {
    row: u8,
    from_left: u8,
    from_right: u8,
}

impl LinkPushRange {
    pub fn at(board: &MahjongLinkBoard, cell: LinkCell) -> Option<Self> {
        let line = board
            .cells()
            .as_chunks::<BOARD_COLUMNS>()
            .0
            .get(usize::from(cell.row))?;
        let column = usize::from(cell.column);
        let from_left = if line.iter().position(Option::is_some) == Some(column) {
            board.maximum_push(cell.row, PushSide::Left)
        } else {
            0
        };
        let from_right = if line.iter().rposition(Option::is_some) == Some(column) {
            board.maximum_push(cell.row, PushSide::Right)
        } else {
            0
        };
        (from_left > 0 || from_right > 0).then_some(Self {
            row: cell.row,
            from_left,
            from_right,
        })
    }

    fn snap(self, displacement: Vec2, column_width: f32) -> Option<(PushSide, u8)> {
        if displacement.x.abs() < displacement.y.abs() {
            return None;
        }
        let columns = (displacement.x / column_width).round();
        let (from, limit) = if columns > 0.0 {
            (PushSide::Left, self.from_left)
        } else {
            (PushSide::Right, self.from_right)
        };
        let distance = columns.abs().min(f32::from(limit)) as u8;
        (distance > 0).then_some((from, distance))
    }
}

pub(super) struct LinkPushPreview {
    pub action: MahjongLinkAction,
    pub movements: Vec<LinkMovement>,
}

pub(super) struct LinkPushGesture {
    pointer: PointerId,
    entity: Entity,
    range: LinkPushRange,
    column_width: f32,
    moved: bool,
    pub preview: Option<LinkPushPreview>,
}

impl LinkPushGesture {
    fn update(&mut self, board: &MahjongLinkBoard, distance: Vec2) {
        self.moved |= distance.length() > self.column_width * 0.2;
        let Some((from, distance)) = self.range.snap(distance, self.column_width) else {
            self.preview = None;
            return;
        };
        let row = self.range.row;
        let action = MahjongLinkAction::Push {
            row,
            from,
            distance,
        };
        if self
            .preview
            .as_ref()
            .is_some_and(|preview| preview.action == action)
        {
            return;
        }
        self.preview = board
            .clone()
            .push(row, from, distance)
            .ok()
            .map(|movements| LinkPushPreview { action, movements });
    }

    fn owns(&self, pointer: PointerId, entity: Entity, button: PointerButton) -> bool {
        self.pointer == pointer && self.entity == entity && button == PointerButton::Primary
    }
}

#[derive(SystemParam)]
pub(super) struct BoardInput<'w, 's> {
    starts: MessageReader<'w, 's, PointerDragStart>,
    drags: MessageReader<'w, 's, PointerDrag>,
    ends: MessageReader<'w, 's, PointerDragEnd>,
    clicks: MessageReader<'w, 's, PointerClick>,
    cancels: MessageReader<'w, 's, PointerCancel>,
    #[cfg(target_os = "android")]
    touches: MessageReader<'w, 's, TouchInput>,
    hitboxes: Query<'w, 's, &'static LinkTileHitbox>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    viewport: Res<'w, MahjongProjectionViewport>,
    scale: Res<'w, UiScale>,
    shell: Res<'w, UiState>,
    confirmation: Res<'w, ConfirmationDialog>,
    motion: Res<'w, PageMotion>,
}

impl BoardInput<'_, '_> {
    fn blocked(&mut self) -> bool {
        // Android adapts touch cancellation to a release, which must not submit a move.
        #[cfg(target_os = "android")]
        let touch_canceled = self
            .touches
            .read()
            .any(|touch| touch.phase == TouchPhase::Canceled);
        #[cfg(not(target_os = "android"))]
        let touch_canceled = false;
        let canceled = self.cancels.read().count() > 0;
        canceled
            || touch_canceled
            || !self.window.single().is_ok_and(|window| window.focused)
            || self.shell.settings.open
            || self.shell.shop.open
            || self.shell.profile.open
            || self.shell.leaving_room
            || self.confirmation.is_open()
            || self.motion.active()
    }

    fn clear(&mut self) {
        self.starts.clear();
        self.drags.clear();
        self.ends.clear();
        self.clicks.clear();
    }
}

pub(super) fn update_board_input(
    mut input: BoardInput,
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<MahjongLinkUiState>,
    mut playback: ResMut<LinkPlayback>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    let blocked = input.blocked();
    let Some(client) = client
        .as_deref_mut()
        .filter(|client| matches!(client.0.state(), NetworkState::Connected(_)))
    else {
        ui.clear_game();
        playback.previewed_push = None;
        input.clear();
        return;
    };
    let model = client.0.model();
    let Some(game) = model.game_snapshot().and_then(GameSnapshot::mahjong_link) else {
        ui.clear_game();
        input.clear();
        return;
    };
    ui.reconcile(game);
    let rejection_serial = model.rejection_serial();
    if ui
        .pending_push
        .as_ref()
        .is_some_and(|(serial, _)| *serial != rejection_serial)
    {
        ui.pending_push = None;
        playback.previewed_push = None;
    }
    if blocked || !can_act(game) || ui.pending_push.is_some() {
        ui.drag = None;
        input.clear();
        return;
    }

    for start in input.starts.read() {
        if game.push_used || ui.drag.is_some() || start.button != PointerButton::Primary {
            continue;
        }
        let Ok(&LinkTileHitbox(cell)) = input.hitboxes.get(start.entity) else {
            continue;
        };
        let Some(range) = LinkPushRange::at(&game.board, cell) else {
            continue;
        };
        let column_width = layout::column_width(cell.row, input.viewport.size) * input.scale.0;
        ui.drag = Some(LinkPushGesture {
            pointer: start.pointer.id,
            entity: start.entity,
            range,
            column_width,
            moved: false,
            preview: None,
        });
    }
    for event in input.drags.read() {
        if let Some(drag) = ui
            .drag
            .as_mut()
            .filter(|drag| drag.owns(event.pointer.id, event.entity, event.button))
        {
            drag.update(&game.board, event.distance);
        }
    }

    let mut dragged_click = None;
    let mut submission = None;
    for end in input.ends.read() {
        if ui
            .drag
            .as_ref()
            .is_some_and(|drag| drag.owns(end.pointer.id, end.entity, end.button))
            && let Some(mut drag) = ui.drag.take()
        {
            drag.update(&game.board, end.distance);
            if drag.moved {
                dragged_click = Some((end.pointer.id, end.entity));
                submission = drag.preview;
            }
        }
    }
    if let Some(preview) = submission {
        let action = preview.action;
        let key = (game.match_id, game.turn, action);
        if client
            .0
            .send(game_command(MahjongLinkCommand::Act { action }))
        {
            ui.pending_push = Some((rejection_serial, preview));
            ui.selected = None;
            playback.previewed_push = Some(key);
        }
        input.clicks.clear();
        return;
    }

    for click in input.clicks.read() {
        if ui.drag.is_none()
            && click.button == PointerButton::Primary
            && dragged_click != Some((click.pointer.id, click.entity))
            && let Ok(&LinkTileHitbox(cell)) = input.hitboxes.get(click.entity)
        {
            actions.write(PressedUiAction(UiAction::MahjongLink(
                MahjongLinkUiAction::Select(cell),
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Entity, LinkCell, LinkMovement, LinkPushGesture, LinkPushRange, MahjongLinkAction,
        MahjongLinkBoard, PointerId, PushSide, Vec2, layout,
    };
    use leocard_mahjong_link::build_deck;

    #[test]
    fn drag_previews_the_authoritative_push_and_can_return_to_the_origin() {
        let mut board = MahjongLinkBoard::new(build_deck()).unwrap();
        let cell = |column| LinkCell { row: 0, column };
        board.remove_pair(cell(14), cell(15)).unwrap();
        assert!(LinkPushRange::at(&board, cell(8)).is_none());
        let mut drag = LinkPushGesture {
            pointer: PointerId::Mouse,
            entity: Entity::PLACEHOLDER,
            range: LinkPushRange::at(&board, cell(16)).unwrap(),
            column_width: layout::column_width(0, Vec2::new(1280.0, 720.0)),
            moved: false,
            preview: None,
        };
        drag.update(&board, Vec2::new(-10.0 * drag.column_width, 0.0));
        let preview = drag.preview.as_ref().unwrap();
        assert_eq!(
            preview.action,
            MahjongLinkAction::Push {
                row: 0,
                from: PushSide::Right,
                distance: 2
            }
        );
        assert_eq!(
            preview.movements,
            vec![LinkMovement {
                from: cell(16),
                to: cell(14)
            }]
        );
        assert!(board.tile(cell(16)).is_some() && board.tile(cell(14)).is_none());
        drag.update(&board, Vec2::ZERO);
        assert!(drag.moved && drag.preview.is_none());
    }
}
