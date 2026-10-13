#[cfg(target_os = "android")]
mod android;
mod plugin;
mod skin;
mod state;
mod systems;
mod view;

pub(crate) use plugin::TextInputPlugin;
pub(crate) use skin::TextInputSkin;
pub(crate) use state::{TextInputAction, TextInputEvent, TextInputSet};
pub(crate) use view::TextInput;
