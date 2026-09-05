use bevy::prelude::*;
use bevy::window::WindowCreated;
use winit::window::Icon;

const APP_ICON_PNG: &[u8] = include_bytes!("../../../../assets/icons/app-icon.png");

pub(super) fn set_app_window_icon(mut created_windows: MessageReader<WindowCreated>) {
    for event in created_windows.read() {
        let icon = decode_app_icon();
        bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
            if let Some(window) = windows.get_window(event.window) {
                window.set_window_icon(Some(icon));
            }
        });
    }
}

fn decode_app_icon() -> Icon {
    let pixels = image::load_from_memory(APP_ICON_PNG)
        .expect("the compiled application icon must be a valid PNG")
        .into_rgba8();
    let (width, height) = pixels.dimensions();
    Icon::from_rgba(pixels.into_raw(), width, height)
        .expect("the compiled application icon must contain RGBA pixels")
}
