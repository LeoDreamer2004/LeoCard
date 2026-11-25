//! Android native entry point and JNI callbacks.

#![cfg(target_os = "android")]

mod callbacks;

#[bevy::prelude::bevy_main]
fn main() {
    leocard_client::run();
}
