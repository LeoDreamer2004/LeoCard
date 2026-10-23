//! Runtime resource discovery and optional single-binary asset data.

mod paths;

pub use paths::asset_file_path;

pub const IS_EMBEDDED: bool = cfg!(leocard_embedded_assets);

include!(concat!(env!("OUT_DIR"), "/embedded_runtime_assets.rs"));
