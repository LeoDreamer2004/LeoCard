//! 不依赖具体游戏的物理牌校验与排名计算。
//!
//! 游戏提供合法牌堆和积分档位，并自行解释规则及校验错误。

mod cards;
mod ranking;

pub use cards::{DeckError, contains_unique_cards, has_unique_cards, validate_deck};
pub use ranking::ranked_awards;
