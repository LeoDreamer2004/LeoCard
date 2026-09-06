//! 通用客户端输入系统。

use bevy::asset::RenderAssetUsages;
use bevy::audio::Volume;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::log::warn;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{FileDragAndDrop, Ime, PrimaryWindow};
use bevy_clipboard::Clipboard;
use leocard_protocol::{ChatContent, ClientCommand, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

use super::*;

mod files;
mod lobby;
mod state;
mod text;
mod ui;

pub use files::*;
pub use lobby::*;
pub use state::*;
pub use text::*;
pub use ui::*;
