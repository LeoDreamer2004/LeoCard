//! 棋牌游戏客户端插件组合。

use super::{mahjong, qigui523, shengji, texas_holdem, uno};
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
        ));
    }
}
