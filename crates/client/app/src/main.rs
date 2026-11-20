#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    #[cfg(feature = "desktop")]
    if leocard_update::run_if_requested() {
        return;
    }
    #[cfg(not(feature = "web"))]
    leocard_client::run();
    #[cfg(feature = "web")]
    wasm_bindgen_futures::spawn_local(async {
        leocard_platform::initialize()
            .await
            .expect("browser storage initialization failed");
        leocard_client::run();
    });
}
