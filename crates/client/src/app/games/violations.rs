//! 游戏拒绝原因到展示文案的领域路由。

use super::{mahjong, qigui523, shengji, texas_holdem, uno};
use leocard_protocol::GameViolation;

pub(crate) fn game_violation_label(violation: &GameViolation) -> Option<String> {
    match violation {
        GameViolation::QiGui523(violation) => {
            qigui523::violation::qigui523_violation_label(violation)
        }
        GameViolation::TexasHoldem(violation) => Some(
            texas_holdem::violation::texas_holdem_violation_label(violation),
        ),
        GameViolation::Shengji(violation) => {
            Some(shengji::violation::shengji_violation_label(violation))
        }
        GameViolation::Uno(violation) => Some(uno::violation::uno_violation_label(violation)),
        GameViolation::Mahjong(violation) => {
            Some(mahjong::violation::mahjong_violation_label(violation))
        }
        _ => None,
    }
}
