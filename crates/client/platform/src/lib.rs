#[cfg(feature = "web")]
mod browser;
mod capabilities;
pub mod storage;

#[cfg(feature = "web")]
pub use browser::{
    allow_browser_back, discard_import, enable_image_drop, initialize, open_directory, open_url,
    request_image, set_room_active, take_browser_back, take_dropped_images,
};
pub use capabilities::*;
pub use web_time as time;
