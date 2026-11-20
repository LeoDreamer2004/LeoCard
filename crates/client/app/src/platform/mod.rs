#[cfg(feature = "android")]
mod android;
#[cfg(feature = "desktop")]
mod desktop;
mod image;
#[cfg(feature = "desktop")]
pub mod notifications;

#[cfg(feature = "android")]
pub use android::*;
#[cfg(feature = "desktop")]
pub use desktop::*;
pub use image::request_image;
#[cfg(feature = "web")]
pub use leocard_platform::{open_directory, open_url};
