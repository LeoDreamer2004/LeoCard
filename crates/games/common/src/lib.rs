//! 不依赖具体游戏的玩家编号、物理牌校验、排名与搜索工具。
//!
//! 游戏提供合法牌堆和积分档位，并自行解释规则及校验错误。

mod cards;
mod player;
mod ranking;
mod search;

pub use cards::{DeckError, contains_unique_cards, has_unique_cards, validate_deck};
pub use player::PlayerId;
pub use ranking::ranked_awards;
pub use search::{SearchBudget, SearchRandom};
