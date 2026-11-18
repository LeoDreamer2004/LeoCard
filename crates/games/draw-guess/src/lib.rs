//! Shared drawing document, rasterization and competitive drawing rounds.

mod drawing;
mod game;
mod library;
mod rules;
mod words;

pub use drawing::{
    CANVAS_HEIGHT, CANVAS_WIDTH, CanvasDocument, CanvasPoint, DrawAction, DrawOperation, Ink,
    RasterCanvas, Stroke,
};
pub use game::{DrawGuessGame, DrawGuessPhase, DrawGuessView, GuessMessage, GuessPlayer};
pub use library::WordLibrary;
pub use rules::{DrawGuessError, DrawGuessRuleSet};
pub use words::word_bank;

#[cfg(test)]
mod tests;
