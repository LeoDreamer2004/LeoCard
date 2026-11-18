use super::{CANVAS_HEIGHT, CANVAS_WIDTH, CanvasDocument, CanvasPoint, DrawOperation, Ink, Stroke};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct RasterCanvas {
    pub pixels: Vec<u8>,
    owners: Vec<u64>,
}

impl Default for RasterCanvas {
    fn default() -> Self {
        let count = usize::from(CANVAS_WIDTH) * usize::from(CANVAS_HEIGHT);
        Self {
            pixels: vec![255; count * 4],
            owners: vec![0; count],
        }
    }
}

impl RasterCanvas {
    pub fn render(&mut self, document: &CanvasDocument) {
        self.pixels.fill(255);
        self.owners.fill(0);
        let erased: HashSet<_> = document
            .operations()
            .iter()
            .filter_map(|op| match op {
                DrawOperation::EraseStroke { id } => Some(*id),
                _ => None,
            })
            .collect();
        for operation in document.operations() {
            match operation {
                DrawOperation::Stroke(stroke) if !erased.contains(&stroke.id) => {
                    self.stroke(stroke)
                }
                DrawOperation::Fill { point, color } => self.fill(*point, *color),
                DrawOperation::Clear => {
                    self.pixels.fill(255);
                    self.owners.fill(0);
                }
                _ => {}
            }
        }
        if let Some(stroke) = document.pending() {
            self.stroke(stroke);
        }
    }

    pub fn stroke_at(&self, point: CanvasPoint) -> Option<u64> {
        if !point.valid() {
            return None;
        }
        let owner =
            self.owners[usize::from(point.y) * usize::from(CANVAS_WIDTH) + usize::from(point.x)];
        (owner != 0).then_some(owner)
    }

    pub fn stroke(&mut self, stroke: &Stroke) {
        self.append_stroke(stroke, 0);
    }

    pub fn append_stroke(&mut self, stroke: &Stroke, from: usize) {
        let Some(first) = stroke.points.first() else {
            return;
        };
        if from == 0 {
            self.segment(*first, *first, stroke);
        }
        for index in from.max(1)..stroke.points.len() {
            self.segment(stroke.points[index - 1], stroke.points[index], stroke);
        }
    }

    fn segment(&mut self, start: CanvasPoint, end: CanvasPoint, stroke: &Stroke) {
        let radius = f32::from(stroke.size) * 0.5;
        let margin = radius.ceil() as i32 + 1;
        let x0 = (i32::from(start.x.min(end.x)) - margin).max(0);
        let x1 = (i32::from(start.x.max(end.x)) + margin).min(i32::from(CANVAS_WIDTH) - 1);
        let y0 = (i32::from(start.y.min(end.y)) - margin).max(0);
        let y1 = (i32::from(start.y.max(end.y)) + margin).min(i32::from(CANVAS_HEIGHT) - 1);
        let dx = f32::from(end.x) - f32::from(start.x);
        let dy = f32::from(end.y) - f32::from(start.y);
        let length = dx * dx + dy * dy;
        let color = if stroke.eraser {
            Ink::WHITE
        } else {
            stroke.color
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 - f32::from(start.x);
                let py = y as f32 - f32::from(start.y);
                let t = if length > 0.0 {
                    ((px * dx + py * dy) / length).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let distance = ((px - t * dx).powi(2) + (py - t * dy).powi(2)).sqrt();
                let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
                if coverage <= 0.0 {
                    continue;
                }
                let index = y as usize * usize::from(CANVAS_WIDTH) + x as usize;
                for channel in 0..3 {
                    let previous = f32::from(self.pixels[index * 4 + channel]);
                    self.pixels[index * 4 + channel] = (previous
                        + (f32::from(color.0[channel]) - previous) * coverage)
                        .round() as u8;
                }
                if coverage >= 0.5 {
                    self.owners[index] = if stroke.eraser { 0 } else { stroke.id };
                }
            }
        }
    }

    fn fill(&mut self, point: CanvasPoint, color: Ink) {
        if !point.valid() {
            return;
        }
        let width = usize::from(CANVAS_WIDTH);
        let start = usize::from(point.y) * width + usize::from(point.x);
        let target = [
            self.pixels[start * 4],
            self.pixels[start * 4 + 1],
            self.pixels[start * 4 + 2],
        ];
        if target == color.0 {
            return;
        }
        let mut queue = vec![start];
        self.paint(start, color);
        while let Some(index) = queue.pop() {
            let x = index % width;
            let neighbors = [
                (x > 0).then(|| index - 1),
                (x + 1 < width).then_some(index + 1),
                (index >= width).then(|| index - width),
                (index + width < self.owners.len()).then_some(index + width),
            ];
            for next in neighbors.into_iter().flatten() {
                if self.pixels[next * 4..next * 4 + 3] == target {
                    self.paint(next, color);
                    queue.push(next);
                }
            }
        }
    }

    fn paint(&mut self, index: usize, color: Ink) {
        self.pixels[index * 4..index * 4 + 3].copy_from_slice(&color.0);
        self.owners[index] = 0;
    }
}
