mod decision;
mod defense;
mod observation;
mod search;
mod shanten;
mod targets;
#[cfg(test)]
mod tests;

pub use decision::{MahjongBot, MahjongBotAction};
pub use observation::MahjongBotObservation;
