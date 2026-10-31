use crate::{TexasCard, TexasHandCategory, TexasStreet};

/// Facts about an accepted action, before street transitions and pot settlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasActionStatistics {
    pub street: TexasStreet,
    /// Total wager on this street for a voluntary all-in, including chips already invested.
    pub all_in_amount: u32,
    pub full_raise: bool,
    pub raised: bool,
    /// Preflop: the big blind is level 1, the first full raise level 2.
    pub bet_level: u32,
}

/// One player's running statistics within the current match.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasMatchStatistics {
    pub hands: u32,
    pub won_hands: u32,
    pub raised_hands: u32,
    pub consecutive_folds: u32,
    /// Longest current suffix of wins with mutually different categories.
    pub consecutive_winning_categories: Vec<TexasHandCategory>,
    pub lowest_starting_stack: u32,
}

/// Recipient-specific settlement facts. Contains no other player's hole cards or ID.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasHandStatistics {
    pub hole_cards: Vec<TexasCard>,
    pub category: Option<TexasHandCategory>,
    /// Actual received chips from pots with at least two contributors; refunds excluded.
    pub won_chips: u32,
    /// Total size of contested pots won, before splitting tied pots.
    pub won_pot: u32,
    pub won_side_pot_chips: u32,
    /// Largest number of tied winners sharing a contested pot received by this player.
    pub tied_winners: u8,
    pub showdown: bool,
    /// Reached the flop without folding, or won uncontested before the flop.
    pub entered_pot: bool,
    pub short_deck: bool,
    pub button: bool,
    pub big_blind: bool,
    pub under_the_gun: bool,
    pub defended_blind: bool,
    pub preflop_bet_levels: Vec<u32>,
    pub river_bet_called: bool,
    pub side_pot_raises: u32,
    pub defeated_categories: Vec<TexasHandCategory>,
    pub defeated_pocket_pairs: u32,
    pub defeated_preflop_all_in_pairs: u32,
    pub preflop_all_in: bool,
    /// Categories folded after this player's final river all-in raise.
    pub river_all_in_folded_categories: Vec<TexasHandCategory>,
    pub final_stack: u32,
    pub total_chips: u32,
    pub match_complete: bool,
    pub first_place: bool,
    pub match_statistics: TexasMatchStatistics,
}
