mod classification;
mod follow;
mod state;
mod structure;
#[cfg(test)]
mod tests;

pub(crate) use classification::classify_cards;
pub use classification::compare_for_trick;
pub(crate) use classification::{category, strength};
pub use follow::{classify_lead, follow_suggestions, forced_follow_cards, validate_follow};
pub use state::*;
