use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const GENERATED_FILE: &str = "embedded_runtime_assets.rs";

fn main() {
    println!("cargo:rustc-check-cfg=cfg(leocard_embedded_assets)");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBEDDED_ASSETS");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    embed_windows_resources(&manifest_dir);
    let runtime_manifest = manifest_dir.join("runtime-assets.txt");
    println!("cargo:rerun-if-changed={}", runtime_manifest.display());

    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(GENERATED_FILE);
    let enabled = env::var_os("CARGO_FEATURE_EMBEDDED_ASSETS").is_some()
        || env::var("PROFILE").is_ok_and(|profile| profile == "release");

    if !enabled {
        fs::write(
            output,
            "fn embedded_asset_dir() -> bevy::asset::io::memory::Dir { unreachable!() }\n",
        )
        .unwrap();
        return;
    }

    println!("cargo:rustc-cfg=leocard_embedded_assets");
    let asset_root = manifest_dir.join("../../assets");
    let entries = fs::read_to_string(&runtime_manifest).unwrap_or_else(|error| {
        panic!(
            "failed to read runtime asset manifest {}: {error}",
            runtime_manifest.display()
        )
    });
    let mut assets = BTreeMap::<String, PathBuf>::new();

    for (index, raw_line) in entries.lines().enumerate() {
        let entry = raw_line.trim();
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        validate_manifest_entry(entry, index + 1);
        let source = asset_root.join(entry);
        if entry.ends_with('/') {
            // 还要监听目录本身，否则向已列出的目录新增资源时，Cargo 不会重跑
            // build script，release 内嵌包会继续沿用缺少新文件的旧生成结果。
            println!("cargo:rerun-if-changed={}", source.display());
            collect_directory(&asset_root, &source, &mut assets);
        } else {
            collect_file(&asset_root, &source, &mut assets);
        }
    }

    assert!(
        !assets.is_empty(),
        "runtime asset manifest selected no files"
    );
    let mut generated = String::from(
        "fn embedded_asset_dir() -> bevy::asset::io::memory::Dir {\n\
         \tlet dir = bevy::asset::io::memory::Dir::default();\n",
    );
    for (asset_path, source) in &assets {
        writeln!(
            generated,
            "\tdir.insert_asset(std::path::Path::new({asset_path:?}), include_bytes!({source:?}).as_slice());",
            source = source.to_string_lossy(),
        )
        .unwrap();
        println!("cargo:rerun-if-changed={}", source.display());
    }
    generated.push_str("\tdir\n}\n");
    fs::write(output, generated).unwrap();
}

fn embed_windows_resources(manifest_dir: &Path) {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let icon = manifest_dir.join("../../assets/icons/app-icon.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    winresource::WindowsResource::new()
        .set_icon(
            icon.to_str()
                .expect("Windows icon path must be valid Unicode"),
        )
        .compile()
        .expect("failed to embed the application icon into the Windows executable");
}

fn validate_manifest_entry(entry: &str, line: usize) {
    let path = Path::new(entry);
    assert!(
        !path.is_absolute()
            && path.components().all(|component| matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )),
        "runtime-assets.txt line {line} must stay below assets/: {entry}"
    );
}

fn collect_directory(asset_root: &Path, directory: &Path, assets: &mut BTreeMap<String, PathBuf>) {
    let mut children = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_directory(asset_root, &child, assets);
        } else if child.is_file() {
            collect_file(asset_root, &child, assets);
        }
    }
}

fn collect_file(asset_root: &Path, source: &Path, assets: &mut BTreeMap<String, PathBuf>) {
    assert!(
        source.is_file(),
        "runtime asset is missing: {}",
        source.display()
    );
    let relative = source.strip_prefix(asset_root).unwrap();
    let asset_path = relative
        .components()
        .map(|component| component.as_os_str())
        .collect::<Vec<&OsStr>>()
        .join(OsStr::new("/"))
        .to_string_lossy()
        .into_owned();
    let previous = assets.insert(asset_path.clone(), source.to_path_buf());
    assert!(
        previous.is_none(),
        "runtime asset selected twice: {asset_path}"
    );
}
