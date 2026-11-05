mod icon;
mod state;
mod systems;
#[cfg(test)]
mod tests;

pub(crate) use icon::{add_egg_thrower_button, add_egg_thrower_icon};
pub(crate) use state::{AutoEggThrower, EggThrowerCount};
pub(crate) use systems::{sync_egg_thrower_counts, tick_auto_egg_thrower};
