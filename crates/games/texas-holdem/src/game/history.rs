use super::{TexasHoldemAction, TexasHoldemPlayerId, TexasHoldemStreet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BettingRecord {
    pub player: TexasHoldemPlayerId,
    pub street: TexasHoldemStreet,
    pub action: TexasHoldemAction,
    pub payment: u32,
    pub pot_before: u32,
}
