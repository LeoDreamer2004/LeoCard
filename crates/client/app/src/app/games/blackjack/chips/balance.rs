use crate::app::presentation::{add_poker_balance_panel, poker_chip_counts};
use crate::app::runtime::UiAssets;
use leocard_ui::UiComposer;

use bevy::prelude::*;

pub(in super::super) fn add_blackjack_balance(
    commands: &mut UiComposer,
    portrait: Entity,
    balance: u32,
    assets: &UiAssets,
) -> Entity {
    add_poker_balance_panel(
        commands,
        portrait,
        Vec2::new(112.0, 90.0),
        balance,
        &poker_chip_counts(balance),
        assets,
    )
}
