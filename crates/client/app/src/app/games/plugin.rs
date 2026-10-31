//! 棋牌游戏客户端插件组合。

use super::doudizhu::DoudizhuPlugin;
use super::{blackjack, mahjong, mc_guess, qigui523, shengji, sync_summary_playback, texas, uno};
use crate::app::presentation::update_summary_animation;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct GamesPlugin;

impl Plugin for GamesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            blackjack::BlackjackPlugin,
            DoudizhuPlugin,
            mc_guess::McGuessPlugin,
            qigui523::QiGui523Plugin,
            texas::TexasPlugin,
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
