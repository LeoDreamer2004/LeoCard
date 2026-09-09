mod assignment;
mod hierarchy;
mod suggestions;
mod validation;

pub(super) use assignment::best_follow_tier;
use assignment::{forced_cards_for_hierarchy, special_follow_card_sets};
use hierarchy::FollowPattern;
#[cfg(test)]
pub(super) use hierarchy::eight_card_tractor_hierarchy;
pub(super) use hierarchy::special_follow_hierarchy;
pub use suggestions::*;
use validation::triple_follow_rank;
pub use validation::*;
