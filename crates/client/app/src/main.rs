#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    #[cfg(not(target_os = "android"))]
    if leocard_update::run_if_requested() {
        return;
    }
    leocard_client::run();
}
