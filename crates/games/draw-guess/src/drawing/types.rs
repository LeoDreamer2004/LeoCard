use serde::{Deserialize, Serialize};
pub const CANVAS_WIDTH: u16 = 960;
pub const CANVAS_HEIGHT: u16 = 600;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CanvasPoint {
    pub x: u16,
    pub y: u16,
}

impl CanvasPoint {
    pub fn valid(self) -> bool {
        self.x < CANVAS_WIDTH && self.y < CANVAS_HEIGHT
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ink(pub [u8; 3]);

impl Ink {
    pub const BLACK: Self = Self([24, 24, 28]);
    pub const WHITE: Self = Self([255; 3]);
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Stroke {
    pub id: u64,
    pub color: Ink,
    pub size: u8,
    pub eraser: bool,
    pub points: Vec<CanvasPoint>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DrawOperation {
    Stroke(Stroke),
    Fill { point: CanvasPoint, color: Ink },
    EraseStroke { id: u64 },
    Clear,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DrawAction {
    Begin {
        point: CanvasPoint,
        color: Ink,
        size: u8,
        eraser: bool,
    },
    Extend {
        points: Vec<CanvasPoint>,
    },
    End,
    Fill {
        point: CanvasPoint,
        color: Ink,
    },
    EraseStroke {
        id: u64,
    },
    Clear,
    Undo,
    Redo,
}
