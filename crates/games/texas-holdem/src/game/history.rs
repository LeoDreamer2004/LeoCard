use super::{TexasAction, TexasStreet};
use leocard_game_common::PlayerId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TexasBettingRecord {
    pub player: PlayerId,
    pub street: TexasStreet,
    pub action: TexasAction,
    pub payment: u32,
    pub pot_before: u32,
}
