use super::*;

mod actions;
mod bidding;
mod cards;
mod hand;
mod play;
mod result;
mod table;

use actions::*;
use bidding::*;
use cards::*;
use hand::*;
use play::*;
use result::*;

pub use actions::select_forced_shengji_follow_cards;
pub use bidding::{shengji_declaration_candidate, sync_shengji_bidding_countdown};
pub use cards::{
    shengji_card_face, shengji_current_level, shengji_display_trump, shengji_hand_sort_trump,
    sort_shengji_cards,
};
pub use hand::{animate_shengji_hand_cards, queue_shengji_deal_animations};
pub use table::{ShengjiTableVisuals, render_shengji_table};

#[cfg(test)]
pub use bidding::shengji_bidding_countdown_label;
#[cfg(test)]
pub use cards::shengji_trump_star_count;
#[cfg(test)]
pub use result::shengji_settlement_outcome_for_score;
#[cfg(test)]
pub use table::shengji_previous_trick_button_state;
