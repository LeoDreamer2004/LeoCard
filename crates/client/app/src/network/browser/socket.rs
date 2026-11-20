use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/network/browser/socket.js")]
extern "C" {
    #[wasm_bindgen(js_name = rememberedToken)]
    pub(super) fn remembered_token(url: &str) -> Option<String>;
    #[wasm_bindgen(js_name = pendingRoom)]
    pub(crate) fn pending_room() -> Option<String>;
    #[wasm_bindgen(js_name = forgetRoom)]
    pub(in crate::network) fn forget_room();
    #[wasm_bindgen(catch, js_name = startConnection)]
    pub(super) fn start(
        url: &str,
        join: &[u8],
        ping: &[u8],
        callback: &Function,
        settings: &str,
    ) -> Result<u32, JsValue>;
    #[wasm_bindgen(js_name = sendFrame)]
    pub(super) fn send(id: u32, frame: &[u8]) -> bool;
    #[wasm_bindgen(js_name = stopConnection)]
    pub(super) fn stop(id: u32);
    #[wasm_bindgen(catch, js_name = queryRoom)]
    pub(super) fn query(url: &str, request: &[u8], callback: &Function) -> Result<u32, JsValue>;
}
