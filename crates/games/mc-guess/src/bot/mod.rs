mod decision;
mod observation;
#[cfg(test)]
mod tests;

pub use decision::{McGuessBot, McGuessBotAction};
pub use observation::McGuessBotObservation;
