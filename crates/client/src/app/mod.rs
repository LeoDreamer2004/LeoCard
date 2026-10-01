//! Bevy 客户端应用。

mod achievements;
mod games;
mod presentation;
mod runtime;
mod shell;
#[cfg(test)]
mod tests;

pub(super) use runtime::launch as run;
