use bevy::prelude::*;
use leocard_protocol::MatchId;

pub(super) struct LogMessage {
    pub name: String,
    pub avatar: Option<Handle<Image>>,
    pub node: u16,
    pub guess: String,
    pub correct: bool,
    pub combo: u16,
}

#[derive(Resource)]
pub(in super::super::super) struct GraphLog {
    pub(super) key: Option<(MatchId, u8)>,
    pub(super) messages: Vec<LogMessage>,
    pub(super) expanded: bool,
    pub(super) version: usize,
    pub(super) scroll_y: f32,
    pub(super) follow_tail: bool,
    pub(super) height: f32,
}

impl Default for GraphLog {
    fn default() -> Self {
        Self {
            key: None,
            messages: Vec::new(),
            expanded: false,
            version: 0,
            scroll_y: 0.0,
            follow_tail: true,
            height: 96.0,
        }
    }
}

impl GraphLog {
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
        self.follow_tail = true;
        self.version += 1;
    }
}

#[derive(Component)]
pub(in super::super::super) struct GraphLogRoot {
    pub(super) key: (MatchId, u8),
    pub(super) version: usize,
    pub(super) layer: Entity,
}

#[derive(Component)]
pub(super) struct GraphLogScroll;

#[derive(Component)]
pub(super) struct GraphLogMessage;
