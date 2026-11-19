use crate::app::runtime::{ConnectionDraft, PageErrorState};
use crate::app::shell::connection::rooms::JoinRoomDialogKind;
use crate::app::shell::{ConnectionUiAction, ModalInputScope, PressedUiAction, UiAction, UiState};
use bevy::prelude::*;
use bevy::text::EditableText;
use leocard_protocol::MAX_PLAYER_NAME_CHARS;
use leocard_ui::{TextInputAction, TextInputEvent};

#[derive(Clone, Copy)]
pub(crate) enum InputField {
    PlayerName,
    HostPort,
    JoinAddress,
    RoomName,
}

impl InputField {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::RoomName => "connection.room-name",
            Self::PlayerName => "connection.name",
            Self::HostPort => "connection.port",
            Self::JoinAddress => "connection.address",
        }
    }

    pub(super) fn maximum_length(self) -> usize {
        match self {
            Self::RoomName => 40,
            Self::PlayerName => MAX_PLAYER_NAME_CHARS,
            Self::HostPort => 5,
            Self::JoinAddress => 64,
        }
    }

    pub(super) fn filter(self) -> fn(char) -> bool {
        match self {
            Self::RoomName | Self::PlayerName => |character| !character.is_control(),
            Self::HostPort => |character| character.is_ascii_digit(),
            Self::JoinAddress => |character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | ':' | '[' | ']' | '-' | ' ')
            },
        }
    }

    pub(super) fn tab_index(self) -> i32 {
        match self {
            Self::RoomName => 0,
            Self::PlayerName => 0,
            Self::HostPort => 1,
            Self::JoinAddress => 2,
        }
    }
}

#[derive(Component)]
pub(super) struct ConnectionField(pub InputField);

pub(super) fn sync_connection_inputs(
    inputs: Query<(&ConnectionField, &EditableText)>,
    mut form: ResMut<ConnectionDraft>,
    mut error: ResMut<PageErrorState>,
    mut ui: ResMut<UiState>,
) {
    for (field, input) in &inputs {
        if matches!(field.0, InputField::RoomName | InputField::JoinAddress) {
            if let Some(editor) = ui.rooms.dialog.as_mut() {
                let value = if matches!(field.0, InputField::RoomName) {
                    &mut editor.name
                } else {
                    &mut editor.address
                };
                if input.value() != value.as_str() {
                    *value = input.value().to_string();
                    error.error = None;
                }
            }
            continue;
        }
        let value = match field.0 {
            InputField::RoomName => continue,
            InputField::PlayerName => &mut form.player_name,
            InputField::HostPort => &mut form.host_port,
            InputField::JoinAddress => &mut form.join_address,
        };
        if input.value() != value.as_str() {
            *value = input.value().to_string();
            error.error = None;
        }
    }
}

pub(super) fn submit_room_inputs(
    mut events: MessageReader<TextInputEvent>,
    fields: Query<&ConnectionField>,
    ui: Res<UiState>,
    scope: ModalInputScope,
    mut actions: MessageWriter<PressedUiAction>,
) {
    for event in events.read() {
        let Some(editor) = ui
            .rooms
            .dialog
            .as_ref()
            .filter(|_| ui.rooms.open && ui.rooms.dialog_open && !ui.rooms.connecting)
        else {
            continue;
        };
        if !scope.allows(event.entity)
            || !fields.get(event.entity).is_ok_and(|field| {
                matches!(field.0, InputField::JoinAddress | InputField::RoomName)
            })
        {
            continue;
        }
        let action = match event.action {
            TextInputAction::Cancel => ConnectionUiAction::CancelRoomEdit,
            TextInputAction::Submit if matches!(editor.kind, JoinRoomDialogKind::Delete(_)) => {
                continue;
            }
            TextInputAction::Submit if matches!(editor.kind, JoinRoomDialogKind::Direct) => {
                ConnectionUiAction::JoinRoom
            }
            TextInputAction::Submit => ConnectionUiAction::SaveRoom,
        };
        actions.write(PressedUiAction(UiAction::Connection(action)));
    }
}
