use std::env;
use std::path::Path;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/icons/app-icon.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    winresource::WindowsResource::new()
        .set_icon(
            icon.to_str()
                .expect("Windows icon path must be valid Unicode"),
        )
        .compile()
        .expect("failed to embed the application icon into the Windows executable");
}
