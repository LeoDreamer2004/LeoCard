mod assets;
mod material;
mod motion;
mod plugin;
mod scene;

use assets::BackgroundAssets;
use material::BackgroundMaterial;
use motion::{FloatingSuit, SUITS, animate_background};
pub(super) use plugin::AmbientBackgroundPlugin;
pub(crate) use scene::add_page_background;
use scene::{setup_background, sync_background_visibility};
