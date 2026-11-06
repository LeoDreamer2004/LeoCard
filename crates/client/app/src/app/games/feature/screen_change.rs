//! 游戏内容是否需要重建页面，由各游戏的比较规则决定。

use leocard_protocol::GameSnapshot;

/// A game implements this only when selected snapshot changes update local visuals in place.
pub(crate) trait GameScreenChange {
    fn is_transient_change(&self, after: &Self) -> bool;
}

pub(crate) struct ScreenChangeCapture {
    before: Option<GameSnapshot>,
}

impl ScreenChangeCapture {
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
                before.is_transient_change(after)
            }
            (Some(GameSnapshot::McGuess(before)), Some(GameSnapshot::McGuess(after))) => {
                before.is_transient_change(after)
            }
            (Some(GameSnapshot::Texas(before)), Some(GameSnapshot::Texas(after))) => {
                before.is_transient_change(after)
            }
            (Some(GameSnapshot::QiGui523(before)), Some(GameSnapshot::QiGui523(after))) => {
                before.is_transient_change(after)
            }
            (Some(GameSnapshot::Shengji(before)), Some(GameSnapshot::Shengji(after))) => {
                before.is_transient_change(after)
            }
            _ => false,
        }
    }
}
