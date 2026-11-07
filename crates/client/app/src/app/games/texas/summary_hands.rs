//! Revealed hand summaries keep each evaluation paired with its own run.
use super::{add_texas_card, texas_category_label};
use crate::app::presentation::{READY, add_animated_summary_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::TexasRevealedHand;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn add_summary_hands(
    commands: &mut UiComposer,
    parent: Entity,
    hand: &TexasRevealedHand,
    twice: bool,
    delay: f32,
    elapsed: f32,
    assets: &UiAssets,
) {
    for best in [hand.best, hand.second_best].into_iter().flatten() {
        let group = spawn_node(
            commands,
            parent,
            Node {
                min_width: px(0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(if twice { 3.0 } else { 7.0 }),
                ..default()
            },
            None,
        );
        add_animated_summary_text(
            commands,
            group,
            texas_category_label(best.category()),
            if twice { 12.0 } else { 14.0 },
            READY,
            delay,
            elapsed,
            assets,
        );
        for card in best.cards() {
            add_texas_card(
                commands,
                group,
                card,
                if twice { (22.0, 32.0) } else { (28.0, 40.0) },
                None,
                assets,
            );
        }
    }
}
