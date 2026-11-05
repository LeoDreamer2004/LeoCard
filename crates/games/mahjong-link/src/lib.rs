//! Deterministic, public-information mahjong linking and row pushing rules.

mod board;
mod game;
mod types;

pub use board::{BOARD_COLUMNS, BOARD_ROWS, MahjongLinkBoard, build_deck, tile_points};
pub use game::MahjongLinkGameState;
pub use types::{
    LinkCell, LinkMovement, LinkPoint, MahjongLinkAction, MahjongLinkError, MahjongLinkOutcome,
    MahjongLinkRuleSet, PushSide,
};

#[cfg(test)]
mod tests;
