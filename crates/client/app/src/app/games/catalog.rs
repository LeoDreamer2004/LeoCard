//! 客户端支持的游戏顺序，名称和简介由各游戏提供。

use super::{GameDescriptor, GameMetadata};
use leocard_blackjack::BlackjackRuleSet;
use leocard_doudizhu::DoudizhuRuleSet;
use leocard_guandan::GuandanRuleSet;
use leocard_mahjong::MahjongRuleSet;
use leocard_mahjong_link::MahjongLinkRuleSet;
use leocard_mc_guess::McGuessRuleSet;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasRuleSet;
use leocard_uno::UnoRuleSet;

pub(crate) const SUPPORTED_GAMES: [GameDescriptor; 10] = [
    QiGuiRuleSet::DESCRIPTOR,
    TexasRuleSet::DESCRIPTOR,
    ShengjiRuleSet::DESCRIPTOR,
    UnoRuleSet::DESCRIPTOR,
    MahjongRuleSet::DESCRIPTOR,
    McGuessRuleSet::DESCRIPTOR,
    BlackjackRuleSet::DESCRIPTOR,
    DoudizhuRuleSet::DESCRIPTOR,
    GuandanRuleSet::DESCRIPTOR,
    MahjongLinkRuleSet::DESCRIPTOR,
];
