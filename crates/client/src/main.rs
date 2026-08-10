#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod updater;

fn main() {
    if updater::run_if_requested() {
        return;
    }
    app::run();
}
