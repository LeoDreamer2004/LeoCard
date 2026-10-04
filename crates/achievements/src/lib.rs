//! Achievement definitions and deterministic progress evaluation.
//! No transport, filesystem, clock, or presentation dependencies.

mod activity;
mod progress;
mod registry;

pub use activity::*;
pub use progress::*;
pub use registry::*;
