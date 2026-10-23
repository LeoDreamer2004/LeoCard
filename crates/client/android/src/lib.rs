//! Android native entry point; desktop builds link only the application rlib.

#[cfg(target_os = "android")]
mod callbacks;

#[cfg(target_os = "android")]
#[bevy::prelude::bevy_main]
fn main() {
    leocard_client::run();
}
