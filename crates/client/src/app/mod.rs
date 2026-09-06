//! Bevy 客户端应用。

mod games;
mod presentation;
mod runtime;
mod shell;

use games::{mahjong::*, qigui523::*, shengji::*, texas_holdem::*, uno::*};
use presentation::*;
use runtime::*;
use shell::*;

pub fn run() {
    launch();
}

#[cfg(test)]
mod tests;
