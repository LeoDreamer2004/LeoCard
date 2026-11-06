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
    for (run, best) in [("第一次", hand.best), ("第二次", hand.second_best)] {
        let Some(best) = best else { continue };
        let group = spawn_node(
            commands,
            parent,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(if twice { 3.0 } else { 7.0 }),
                ..default()
            },
            None,
        );
        let category = texas_category_label(best.category());
        add_animated_summary_text(
            commands,
            group,
            if twice {
                format!("{run}\n{category}")
            } else {
                category.to_owned()
            },
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
