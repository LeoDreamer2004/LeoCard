//! 游戏内容是否需要重建页面，由各游戏的比较规则决定。
use super::blackjack::only_blackjack_timer_changed;
use super::mc_guess::only_guess_timer_changed;
use super::qigui523::only_turn_timer_changed;
use super::shengji::only_shengji_transient_progress_changed;
use super::texas::only_texas_equities_changed;
use leocard_protocol::GameSnapshot;

pub(crate) struct GameScreenChange {
    before: Option<GameSnapshot>,
}
impl GameScreenChange {
    pub(crate) fn capture(game: Option<&GameSnapshot>) -> Self {
        Self {
            before: game
                .filter(|game| {
                    matches!(
                        game,
                        GameSnapshot::QiGui523(_)
                            | GameSnapshot::Shengji(_)
                            | GameSnapshot::Texas(_)
                            | GameSnapshot::McGuess(_)
                            | GameSnapshot::Blackjack(_)
                    )
                })
                .cloned(),
        }
    }

    pub(crate) fn is_transient(&self, after: Option<&GameSnapshot>) -> bool {
        match (self.before.as_ref(), after) {
            (Some(GameSnapshot::Blackjack(before)), Some(GameSnapshot::Blackjack(after))) => {
                only_blackjack_timer_changed(before, after)
            }
            (Some(GameSnapshot::McGuess(before)), Some(GameSnapshot::McGuess(after))) => {
                only_guess_timer_changed(before, after)
            }
            (Some(GameSnapshot::Texas(before)), Some(GameSnapshot::Texas(after))) => {
                only_texas_equities_changed(before, after)
            }
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
