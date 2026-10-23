//! Release downloads, checksum verification and desktop update installation.

mod download;
mod event;
#[cfg(not(target_os = "android"))]
mod installer;
#[cfg(not(target_os = "android"))]
mod receipt;
#[cfg(all(test, not(target_os = "android")))]
mod tests;

pub use download::{download_latest_release, format_bytes};
pub use event::UpdateEvent;
#[cfg(not(target_os = "android"))]
pub use installer::{launch_installer, run_if_requested};
#[cfg(not(target_os = "android"))]
pub use receipt::completed_update;
