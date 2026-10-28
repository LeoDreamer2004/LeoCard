mod bronze;
mod catalog;
mod events;
mod facts;
mod gold;
mod macros;
mod silver;
mod tracking;
mod types;

pub use catalog::{DEFINITIONS, register};
pub use events::DoudizhuAchievementEvent;
pub use tracking::{DoudizhuAchievementTracker, DoudizhuActionContext};
pub use types::{
    DoudizhuAchievementFacts, DoudizhuHandFacts, DoudizhuMatchFacts, DoudizhuPlayFacts,
};

#[cfg(test)]
mod tests;
