use super::camera::GraphCamera;
use super::history::{GraphHistory, PanelPose};
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use leocard_protocol::MatchId;

pub(in super::super) struct GraphUiState {
    pub key: Option<(MatchId, u8)>,
    pub positions: Vec<Vec2>,
    pub split: f32,
    pub selected: Option<u16>,
    pub history: GraphHistory,
    pub selection_wait: f32,
    pub seen_guesses: usize,
    pub gesture: Option<Gesture>,
    pub camera: GraphCamera,
    pub intro: f32,
}

impl Default for GraphUiState {
    fn default() -> Self {
        Self {
            key: None,
            positions: Vec::new(),
            split: 0.44,
            selected: None,
            history: GraphHistory::default(),
            selection_wait: 0.0,
            seen_guesses: 0,
            gesture: None,
            camera: GraphCamera::default(),
            intro: 0.0,
        }
    }
}

pub(in super::super) struct Gesture {
    pub pointer: PointerId,
    pub target: GraphDragTarget,
    pub start: Vec2,
    pub anchor: Vec2,
    pub pan: Vec2,
    pub moved: bool,
}

#[derive(Component, Clone, Copy)]
pub(in super::super) enum GraphDragTarget {
    Node(u16),
    Divider,
    Pan,
}

#[derive(Component)]
pub(in super::super) struct GraphCanvas;

#[derive(Component)]
pub(in super::super) struct GraphWorkspace;

#[derive(Component)]
pub(in super::super) struct GraphPane;

#[derive(Component)]
pub(in super::super) struct GraphHistoryPanel {
    pub number: u16,
    pub revision: u64,
    pub outgoing: bool,
}

#[derive(Component, Default)]
pub(in super::super) struct GraphPanelAnimation {
    pub(super) applied: Option<(u64, PanelPose, Vec2)>,
}

#[derive(Component)]
pub(in super::super) struct GraphCurve(pub usize);

#[derive(Component)]
pub(in super::super) struct GraphEdgeLabel(pub usize);

#[derive(Component)]
pub(in super::super) struct GraphNodeVisual {
    pub number: u16,
    pub highlight: f32,
    pub hover: f32,
    pub reveal: f32,
    pub combo: f32,
}

#[derive(Resource, Default)]
pub(in super::super) struct GraphBroadcast {
    pub key: Option<(MatchId, u8)>,
    pub target: Option<(u8, u16)>,
    pub started_at: f64,
}
