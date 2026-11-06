mod choreography;
mod effects;
mod icon;
mod menu;
mod state;
mod systems;

pub(super) use effects::add_egg_barrage;
pub(crate) use effects::animate_egg_barrages;
pub(crate) use icon::add_egg_thrower_icon;
pub(crate) use menu::{add_shoe_button, begin_shoe_hold, click_shoe_button, update_shoe_holds};
pub(crate) use state::{AutoEggThrower, EggThrowerCount};
pub(crate) use systems::{sync_egg_thrower_counts, tick_auto_egg_thrower};
