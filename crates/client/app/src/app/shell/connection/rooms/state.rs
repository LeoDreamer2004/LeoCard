use bevy::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum RoomSelection {
    Saved(String),
    Lan(u64),
}

#[derive(Clone)]
pub(crate) enum JoinRoomDialogKind {
    Add,
    Edit(String),
    Direct,
    Delete(String),
}

#[derive(Clone)]
pub(crate) struct JoinRoomDialog {
    pub kind: JoinRoomDialogKind,
    pub name: String,
    pub address: String,
}

impl JoinRoomDialog {
    pub fn original_address(&self) -> Option<&str> {
        match &self.kind {
            JoinRoomDialogKind::Edit(address) | JoinRoomDialogKind::Delete(address) => {
                Some(address)
            }
            _ => None,
        }
    }
}

#[derive(Default)]
pub(crate) struct JoinRoomUiState {
    pub open: bool,
    pub generation: u64,
    pub scroll_y: f32,
    pub selected: Option<RoomSelection>,
    pub dialog: Option<JoinRoomDialog>,
    pub dialog_open: bool,
    pub dialog_progress: f32,
    pub connecting: bool,
    pub last_click: Option<(RoomSelection, f64)>,
}

impl JoinRoomUiState {
    pub fn set_open(&mut self, open: bool) {
        self.open = open;
        self.dialog = None;
        self.dialog_open = false;
        self.dialog_progress = 0.0;
        self.connecting = false;
        self.last_click = None;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn show_dialog(&mut self, dialog: JoinRoomDialog) {
        self.dialog = Some(dialog);
        self.dialog_open = true;
        self.dialog_progress = 0.0;
    }
}

#[derive(Component)]
pub(crate) struct JoinRoomList;

#[derive(Component, Clone, Copy)]
pub(crate) enum RoomSignal {
    Checking,
    Offline,
    Online(u32),
}

#[derive(Component)]
pub(crate) struct LanSearchPulse(pub usize);
