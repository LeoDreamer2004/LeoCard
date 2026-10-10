#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod installer;
#[cfg(not(target_os = "android"))]
mod receipt;
#[cfg(all(test, not(target_os = "android")))]
mod tests;

#[cfg(target_os = "android")]
pub(crate) use android::launch_installer;
#[cfg(not(target_os = "android"))]
pub(crate) use installer::{launch_installer, run_if_requested};
#[cfg(not(target_os = "android"))]
pub(crate) use receipt::completed_update;
