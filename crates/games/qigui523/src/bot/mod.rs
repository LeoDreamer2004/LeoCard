mod decision;
mod greedy;
mod observation;
#[cfg(test)]
mod tests;

pub use greedy::{QiGui523Bot, QiGui523BotRequest, has_legal_response};
pub use observation::QiGui523BotObservation;
