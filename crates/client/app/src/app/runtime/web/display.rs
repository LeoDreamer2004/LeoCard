use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(raw_module = "../display.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = fullscreenSupported)]
    pub(in crate::app::runtime) fn fullscreen_supported() -> bool;
    #[wasm_bindgen(js_name = isFullscreen)]
    pub(in crate::app::runtime) fn is_fullscreen() -> bool;
    #[wasm_bindgen(js_name = toggleFullscreen)]
    pub(in crate::app::runtime) fn toggle_fullscreen();
}
