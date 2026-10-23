//! UNO 事件驱动音效。

mod assets;
mod plan;
mod state;
mod systems;

pub(crate) use assets::*;
use plan::*;
pub(crate) use state::*;
pub(crate) use systems::*;

#[cfg(test)]
mod tests;
