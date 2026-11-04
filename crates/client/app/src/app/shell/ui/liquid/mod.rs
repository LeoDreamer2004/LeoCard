mod material;
mod motion;
mod plugin;
mod surface;

use material::LiquidMaterial;
use motion::{animate_liquid, capture_ripples, initialize_liquid};
pub(super) use plugin::LiquidUiPlugin;
use surface::LiquidUvRect;
pub(crate) use surface::{LiquidKind, LiquidSurface};
