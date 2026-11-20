//! 将浏览器已下载的资源包交给 Bevy，避免逐文件 HTTP 请求。

use bevy::asset::io::memory::Dir;
use js_sys::{Array, Uint8Array};
use std::path::Path;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;

// The generated bindings live in pkg/, beside the separately packed WASM.
#[wasm_bindgen(raw_module = "../assets.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = takeAssets)]
    fn take_assets() -> Array;
}

pub(in crate::app::runtime) fn asset_directory() -> Dir {
    let directory = Dir::default();
    for entry in take_assets().iter() {
        let entry = Array::from(&entry);
        let path = entry
            .get(0)
            .as_string()
            .expect("web resource packs contain string paths");
        let bytes: Uint8Array = entry.get(1).unchecked_into();
        directory.insert_asset(Path::new(&path), bytes.to_vec());
    }
    directory
}
