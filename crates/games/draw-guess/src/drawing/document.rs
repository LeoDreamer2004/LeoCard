use super::{DrawAction, DrawOperation, Stroke};
use crate::DrawGuessError;

const MAX_OPERATIONS: usize = 1024;
const MAX_POINTS: usize = 32_768;
const MAX_STROKE_POINTS: usize = 4096;
const MAX_BATCH_POINTS: usize = 64;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanvasDocument {
    operations: Vec<DrawOperation>,
    cursor: usize,
    pending: Option<Stroke>,
    next_id: u64,
    revision: u64,
}

impl CanvasDocument {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn operations(&self) -> &[DrawOperation] {
        &self.operations[..self.cursor]
    }

    pub fn pending(&self) -> Option<&Stroke> {
        self.pending.as_ref()
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0 || self.pending.is_some()
    }

    pub fn can_redo(&self) -> bool {
        self.pending.is_none() && self.cursor < self.operations.len()
    }

    pub fn apply(&mut self, action: &DrawAction) -> Result<(), DrawGuessError> {
        if self.pending.is_some()
            && !matches!(
                action,
                DrawAction::Extend { .. } | DrawAction::End | DrawAction::Undo
            )
        {
            return Err(DrawGuessError::StrokeInProgress);
        }
        match action {
            DrawAction::Begin {
                point,
                color,
                size,
                eraser,
            } => {
                if !point.valid() || !(1..=64).contains(size) {
                    return Err(DrawGuessError::InvalidDrawing);
                }
                self.check_capacity(1)?;
                self.operations.truncate(self.cursor);
                self.next_id += 1;
                self.pending = Some(Stroke {
                    id: self.next_id,
                    color: *color,
                    size: *size,
                    eraser: *eraser,
                    points: vec![*point],
                });
            }
            DrawAction::Extend { points } => {
                if points.is_empty()
                    || points.len() > MAX_BATCH_POINTS
                    || points.iter().any(|p| !p.valid())
                {
                    return Err(DrawGuessError::InvalidDrawing);
                }
                self.check_capacity(points.len())?;
                let stroke = self.pending.as_mut().ok_or(DrawGuessError::NoStroke)?;
                if stroke.points.len() + points.len() > MAX_STROKE_POINTS {
                    return Err(DrawGuessError::DrawingLimit);
                }
                stroke.points.extend_from_slice(points);
            }
            DrawAction::End => {
                let stroke = self.pending.take().ok_or(DrawGuessError::NoStroke)?;
                self.push(DrawOperation::Stroke(stroke));
            }
            DrawAction::Fill { point, color } => {
                if !point.valid() {
                    return Err(DrawGuessError::InvalidDrawing);
                }
                self.check_capacity(0)?;
                self.push(DrawOperation::Fill {
                    point: *point,
                    color: *color,
                });
            }
            DrawAction::EraseStroke { id } => {
                let exists = self.operations().iter().any(|operation| matches!(operation, DrawOperation::Stroke(stroke) if stroke.id == *id));
                let erased = self.operations().iter().any(|operation| matches!(operation, DrawOperation::EraseStroke { id: erased } if erased == id));
                if !exists || erased {
                    return Err(DrawGuessError::NoStroke);
                }
                self.check_capacity(0)?;
                self.push(DrawOperation::EraseStroke { id: *id });
            }
            DrawAction::Clear => {
                self.check_capacity(0)?;
                self.push(DrawOperation::Clear);
            }
            DrawAction::Undo => {
                if self.pending.is_some() {
                    let stroke = self.pending.take().unwrap();
                    self.push(DrawOperation::Stroke(stroke));
                }
                if self.cursor == 0 {
                    return Err(DrawGuessError::NothingToUndo);
                }
                self.cursor -= 1;
            }
            DrawAction::Redo => {
                if !self.can_redo() {
                    return Err(DrawGuessError::NothingToRedo);
                }
                self.cursor += 1;
            }
        }
        self.revision += 1;
        Ok(())
    }

    fn push(&mut self, operation: DrawOperation) {
        self.operations.truncate(self.cursor);
        self.operations.push(operation);
        self.cursor += 1;
    }

    fn check_capacity(&self, extra_points: usize) -> Result<(), DrawGuessError> {
        let points: usize = self
            .operations()
            .iter()
            .map(|op| match op {
                DrawOperation::Stroke(stroke) => stroke.points.len(),
                _ => 0,
            })
            .sum();
        let pending = self
            .pending
            .as_ref()
            .map_or(0, |stroke| stroke.points.len());
        if self.cursor >= MAX_OPERATIONS || points + pending + extra_points > MAX_POINTS {
            return Err(DrawGuessError::DrawingLimit);
        }
        Ok(())
    }
}
