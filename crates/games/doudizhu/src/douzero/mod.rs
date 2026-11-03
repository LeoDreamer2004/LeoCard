//! DouZero-WP robots. Human takeover and hints remain in leocard-doudizhu.

mod actions;
mod model;
mod observation;
mod policy;
#[cfg(test)]
mod tests;

pub use observation::DouzeroObservation;
pub use policy::DouzeroBot;
