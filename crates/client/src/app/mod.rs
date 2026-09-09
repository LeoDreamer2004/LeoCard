//! Bevy 客户端应用。

mod games;
mod presentation;
mod runtime;
mod shell;
#[cfg(test)]
mod tests;

use games::*;
use presentation::*;
use runtime::*;
use shell::*;

pub(super) fn run() {
    launch();
}
