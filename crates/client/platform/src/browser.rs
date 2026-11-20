use js_sys::{Array, Promise, Uint8Array};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/src/browser/files.js")]
extern "C" {
    #[wasm_bindgen(js_name = initializeDrop)]
    fn initialize_drop();
    #[wasm_bindgen(js_name = enableImageDrop)]
    pub fn enable_image_drop(enabled: bool);
    #[wasm_bindgen(js_name = takeDroppedFiles)]
    fn take_dropped_files() -> Array;
    #[wasm_bindgen(js_name = chooseImage)]
    fn choose_image(title: &str) -> Promise;
    #[wasm_bindgen(catch, js_name = openUrl)]
    fn launch_url(url: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = openDirectory)]
    fn launch_directory(path: &str) -> Result<(), JsValue>;
}

pub type ImagePickerReceiver = Mutex<mpsc::Receiver<Result<Option<PathBuf>, String>>>;

pub fn request_image(title: &str) -> Result<ImagePickerReceiver, String> {
    let promise = choose_image(title);
    let (sender, receiver) = mpsc::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let result = JsFuture::from(promise)
            .await
            .map(|value| value.as_string().map(PathBuf::from))
            .map_err(|error| format!("选择图片失败：{error:?}"));
        let _ = sender.send(result);
    });
    Ok(Mutex::new(receiver))
}

pub fn open_url(url: &str) -> Result<(), String> {
    launch_url(url).map_err(|error| format!("打开网页失败：{error:?}"))
}

pub fn open_directory(path: &Path) -> Result<(), String> {
    launch_directory(&path.to_string_lossy())
        .map_err(|error| format!("打开文件列表失败：{error:?}"))
}

#[wasm_bindgen(module = "/src/browser/storage.js")]
extern "C" {
    #[wasm_bindgen(js_name = initializeStorage)]
    fn initialize_storage() -> Promise;
    #[wasm_bindgen(js_name = isMobile)]
    pub(crate) fn is_mobile() -> bool;
    #[wasm_bindgen(catch, js_name = readFile)]
    fn read(path: &str) -> Result<Option<Uint8Array>, JsValue>;
    #[wasm_bindgen(catch, js_name = writeFile)]
    fn write(path: &str, bytes: &[u8]) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = renameFile)]
    fn rename(from: &str, to: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = removeFile)]
    fn remove(path: &str) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = listFiles)]
    fn list(path: &str) -> Result<Array, JsValue>;
    #[wasm_bindgen(js_name = fileExists)]
    pub(crate) fn file_exists(path: &str) -> bool;
}

pub async fn initialize() -> Result<(), String> {
    JsFuture::from(initialize_storage())
        .await
        .map_err(|error| format!("无法打开浏览器存储：{error:?}"))?;
    initialize_lifecycle();
    initialize_drop();
    Ok(())
}

pub fn take_dropped_images() -> Vec<PathBuf> {
    take_dropped_files()
        .iter()
        .filter_map(|value| value.as_string().map(PathBuf::from))
        .collect()
}

pub fn discard_import(path: &Path) {
    if path.starts_with("imports") {
        let _ = remove_file(&path.to_string_lossy());
    }
}

pub(crate) fn read_file(path: &str) -> io::Result<Vec<u8>> {
    read(path)
        .map_err(io_error)?
        .map(|bytes| bytes.to_vec())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path))
}

pub(crate) fn write_file(path: &str, bytes: &[u8]) -> io::Result<()> {
    write(path, bytes).map_err(io_error)
}

pub(crate) fn rename_file(from: &str, to: &str) -> io::Result<()> {
    if !file_exists(from) {
        return Err(io::Error::new(io::ErrorKind::NotFound, from));
    }
    rename(from, to).map_err(io_error)
}

pub(crate) fn remove_file(path: &str) -> io::Result<()> {
    match remove(path).map_err(io_error)? {
        true => Ok(()),
        false => Err(io::Error::new(io::ErrorKind::NotFound, path)),
    }
}

pub(crate) fn list_files(path: &str) -> io::Result<Vec<PathBuf>> {
    Ok(list(path)
        .map_err(io_error)?
        .iter()
        .filter_map(|value| value.as_string().map(PathBuf::from))
        .collect())
}

fn io_error(error: JsValue) -> io::Error {
    io::Error::other(error.as_string().unwrap_or_else(|| format!("{error:?}")))
}

#[wasm_bindgen(module = "/src/browser/lifecycle.js")]
extern "C" {
    #[wasm_bindgen(js_name = initializeLifecycle)]
    fn initialize_lifecycle();
    #[wasm_bindgen(js_name = takeBackRequest)]
    pub fn take_browser_back() -> bool;
    #[wasm_bindgen(js_name = allowBack)]
    pub fn allow_browser_back();
    #[wasm_bindgen(js_name = setRoomActive)]
    pub fn set_room_active(active: bool);
}
