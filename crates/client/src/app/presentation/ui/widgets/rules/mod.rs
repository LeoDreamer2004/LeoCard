//! 各游戏共用布局的规则配置控件。

use bevy::prelude::*;
use leocard_mahjong::MahjongRuleSet;
use leocard_qigui523::{QiGuiRuleSet, SuitComparison, TimeControl};
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;

use super::*;

mod config;
mod labels;

pub use config::*;
pub use labels::*;
