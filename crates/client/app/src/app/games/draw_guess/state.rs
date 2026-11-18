use bevy::prelude::*;
use leocard_draw_guess::{CanvasPoint, DrawGuessPhase, Ink};
use leocard_protocol::{DrawGuessSnapshot, MatchId};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DrawTool {
    #[default]
    Pen,
    AreaEraser,
    StrokeEraser,
    Fill,
}

#[derive(Resource)]
pub(super) struct DrawGuessUiState {
    pub key: Option<(MatchId, u32)>,
    pub input: String,
    pub submitted: Option<String>,
    pub last_guess_serial: u32,
    pub tool: DrawTool,
    pub color: Ink,
    pub size: u8,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub drawing: bool,
    pub last_point: Option<CanvasPoint>,
    pub queued_points: Vec<CanvasPoint>,
    pub send_elapsed: f32,
    pub picker_drag: Option<Entity>,
}

impl Default for DrawGuessUiState {
    fn default() -> Self {
        Self {
            key: None,
            input: String::new(),
            submitted: None,
            last_guess_serial: 0,
            tool: DrawTool::Pen,
            color: Ink::BLACK,
            size: 6,
            hue: 0.0,
            saturation: 0.0,
            value: 0.1,
            drawing: false,
            last_point: None,
            queued_points: Vec::new(),
            send_elapsed: 0.0,
            picker_drag: None,
        }
    }
}

impl DrawGuessUiState {
    pub fn select_color(&mut self, color: Ink) {
        self.color = color;
        let [r, g, b] = color.0.map(|channel| f32::from(channel) / 255.0);
        let high = r.max(g).max(b);
        let low = r.min(g).min(b);
        let difference = high - low;
        self.value = high;
        self.saturation = if high > 0.0 { difference / high } else { 0.0 };
        if difference > 0.0 {
            self.hue = (if high == r {
                (g - b) / difference
            } else if high == g {
                (b - r) / difference + 2.0
            } else {
                (r - g) / difference + 4.0
            })
            .rem_euclid(6.0)
                / 6.0;
        }
    }

    pub fn reconcile(&mut self, game: &DrawGuessSnapshot) {
        let key = Some((game.match_id, game.state.turn));
        if self.key != key {
            self.key = key;
            self.input.clear();
            self.submitted = None;
            self.last_guess_serial = 0;
            self.drawing = false;
            self.last_point = None;
            self.queued_points.clear();
            self.picker_drag = None;
        }
        if !can_draw(game) {
            self.drawing = false;
            self.queued_points.clear();
        }
    }

    pub fn set_hsv(&mut self) {
        let hue = self.hue * 6.0;
        let chroma = self.value * self.saturation;
        let x = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
        let (r, g, b) = match hue as u8 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let base = self.value - chroma;
        self.color = Ink([
            ((r + base) * 255.0).round() as u8,
            ((g + base) * 255.0).round() as u8,
            ((b + base) * 255.0).round() as u8,
        ]);
    }
}

pub(super) fn can_draw(game: &DrawGuessSnapshot) -> bool {
    game.state.phase == DrawGuessPhase::Drawing && game.state.artist == Some(game.you)
}

pub(super) fn can_guess(game: &DrawGuessSnapshot) -> bool {
    can_message(game)
        && game
            .state
            .players
            .iter()
            .any(|player| player.id == game.you && !player.guessed)
}

pub(super) fn can_message(game: &DrawGuessSnapshot) -> bool {
    game.state.phase == DrawGuessPhase::Drawing
        && game.state.artist != Some(game.you)
        && game
            .state
            .players
            .iter()
            .any(|player| player.id == game.you && player.connected)
}

#[derive(Component)]
pub(super) struct GuessInput;

#[derive(Component)]
pub(super) struct CanvasPaper;

#[derive(Component)]
pub(super) struct BrushCursor;

#[derive(Component)]
pub(super) struct GuessHistory;

#[derive(Component, Default)]
pub(super) struct HistoryScroll {
    pub serial: u32,
}

#[derive(Component)]
pub(super) struct DrawingClock;
