//! 将当前游戏快照投影为公共结算演出描述。

use super::{mahjong, qigui523, texas_holdem, uno};
use crate::app::presentation::SummaryDescriptor;
use leocard_protocol::GameSnapshot;

pub(crate) fn game_summary_descriptor(game: &GameSnapshot) -> Option<SummaryDescriptor> {
    match game {
        GameSnapshot::QiGui523(game) => qigui523::summary::qigui523_summary_descriptor(game),
        GameSnapshot::TexasHoldem(game) => {
            texas_holdem::settlement::texas_holdem_summary_descriptor(game)
        }
        GameSnapshot::Shengji(_) => None,
        GameSnapshot::Uno(game) => uno::settlement::uno_summary_descriptor(game),
        GameSnapshot::Mahjong(game) => mahjong::settlement::mahjong_summary_descriptor(game),
    }
}
