use crate::app::runtime::{ConnectionDraft, PageErrorState};
use bevy::{prelude::*, text::EditableText};
use leocard_protocol::MAX_PLAYER_NAME_CHARS;

#[derive(Clone, Copy)]
pub(crate) enum InputField {
    PlayerName,
    HostPort,
    JoinAddress,
}

impl InputField {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::PlayerName => "connection.name",
            Self::HostPort => "connection.port",
            Self::JoinAddress => "connection.address",
        }
    }

    pub(super) fn maximum_length(self) -> usize {
        match self {
            Self::PlayerName => MAX_PLAYER_NAME_CHARS,
            Self::HostPort => 5,
            Self::JoinAddress => 64,
        }
    }

    pub(super) fn filter(self) -> fn(char) -> bool {
        match self {
            Self::PlayerName => |character| !character.is_control(),
            Self::HostPort => |character| character.is_ascii_digit(),
            Self::JoinAddress => |character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | ':' | '[' | ']' | '-' | ' ')
            },
        }
    }

    pub(super) fn tab_index(self) -> i32 {
        match self {
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
) {
    for (field, input) in &inputs {
        let value = match field.0 {
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
