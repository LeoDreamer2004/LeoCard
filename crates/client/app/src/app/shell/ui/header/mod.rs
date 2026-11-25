mod game;
#[cfg(any(feature = "android", feature = "web"))]
mod input;
mod motion;
mod page;
mod state;

pub(super) use motion::GameHeaderPlugin;
pub(in crate::app::shell) use page::Header;
