mod decision;
mod defense;
#[cfg(not(target_arch = "wasm32"))]
mod neural;
mod observation;
mod search;
mod shanten;
mod targets;
#[cfg(test)]
mod tests;

pub use decision::{MahjongBot, MahjongBotAction};
#[cfg(not(target_arch = "wasm32"))]
pub use neural::{KdensBot, MahjongModelHistory, MahjongModelObservation};
pub use observation::MahjongBotObservation;
