//! 棋牌游戏客户端插件组合。

use super::summary::sync_summary_playback;
use super::{mahjong, qigui523, shengji, texas_holdem, uno};
use crate::app::presentation::update_summary_animation;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct GamesPlugin;

impl Plugin for GamesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            qigui523::QiGui523Plugin,
            texas_holdem::TexasHoldemPlugin,
            shengji::ShengjiPlugin,
            uno::UnoPlugin,
            mahjong::MahjongPlugin,
        ))
        .add_systems(
            Update,
            sync_summary_playback
                .in_set(ClientUpdateSet::Sync)
                .before(update_summary_animation),
        );
    }
}
