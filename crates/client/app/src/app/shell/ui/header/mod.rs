mod game;
#[cfg(target_os = "android")]
mod input;
mod motion;
mod page;
mod state;

pub(super) use motion::GameHeaderPlugin;
pub(in crate::app::shell) use page::Header;
