//! 玩家档案界面与各游戏长期统计。

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
pub use leocard_client::{LocalPlayerProfile, PlayerRatingProfile};
use leocard_protocol::{
    PlayerGameProfiles, PlayerInteractionKind, PlayerInteractionStats, QiGui523ProfileStats,
    ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};

use super::*;

mod rating;
mod state;
mod stats;
mod view;

pub use rating::*;
pub use state::*;
pub use stats::*;
pub use view::*;
