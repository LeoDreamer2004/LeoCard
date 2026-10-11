#[cfg(target_os = "android")]
mod android;
mod plugin;
mod retention;
mod skin;
mod state;
mod systems;
#[cfg(test)]
mod tests;
mod view;

pub(crate) use plugin::TextInputPlugin;
pub(crate) use retention::TextInputRetention;
pub(crate) use skin::TextInputSkin;
pub(crate) use state::{TextInputAction, TextInputEvent, TextInputSet};
pub(crate) use view::TextInput;
