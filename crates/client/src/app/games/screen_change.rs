//! 游戏内容是否需要重建页面，由各游戏的比较规则决定。
use super::{qigui523::only_turn_timer_changed, shengji::only_shengji_transient_progress_changed};
use leocard_protocol::GameSnapshot;

pub(crate) struct GameScreenChange {
    before: Option<GameSnapshot>,
}
impl GameScreenChange {
    pub(crate) fn capture(game: Option<&GameSnapshot>) -> Self {
        Self {
            before: game
                .filter(|game| matches!(game, GameSnapshot::QiGui523(_) | GameSnapshot::Shengji(_)))
                .cloned(),
        }
    }
    pub(crate) fn is_transient(&self, after: Option<&GameSnapshot>) -> bool {
        match (self.before.as_ref(), after) {
            (Some(GameSnapshot::QiGui523(before)), Some(GameSnapshot::QiGui523(after))) => {
                only_turn_timer_changed(Some(before), Some(after))
            }
            (Some(GameSnapshot::Shengji(before)), Some(GameSnapshot::Shengji(after))) => {
                only_shengji_transient_progress_changed(Some(before), Some(after))
            }
            _ => false,
        }
    }
}
