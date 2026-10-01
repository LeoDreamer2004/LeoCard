//! Achievement definitions and deterministic progress evaluation.
//! No transport, filesystem, clock, or presentation dependencies.

mod progress;
mod registry;

pub use progress::*;
pub use registry::*;
