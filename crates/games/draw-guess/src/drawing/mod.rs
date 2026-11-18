mod document;
mod raster;
mod types;

pub use document::CanvasDocument;
pub use raster::RasterCanvas;
pub use types::{CANVAS_HEIGHT, CANVAS_WIDTH, CanvasPoint, DrawAction, DrawOperation, Ink, Stroke};
