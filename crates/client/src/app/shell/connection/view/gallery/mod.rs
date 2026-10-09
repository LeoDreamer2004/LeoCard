mod assets;
mod design;
mod glow;
mod motion;
mod plugin;
mod scene;

use assets::{GalleryAssets, load_entry_art};
use design::ENTRY_DESIGNS;
use glow::{EntryGlow, GlowKind};
use motion::animate_entries;
use scene::{EntryArt, GameEntry};

pub(in crate::app::shell::connection) use plugin::GameGalleryPlugin;
pub(super) use scene::GameGallery;
