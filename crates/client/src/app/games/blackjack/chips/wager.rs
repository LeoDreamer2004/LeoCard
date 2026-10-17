use super::super::playback::SETTLEMENT_CHIP_DURATION;
use crate::app::presentation::{
    DANGER, MUTED, PokerChipScatter, READY, TablePieceDeal, TableStatusEffect, TableStatusSpec,
    UiComposer, UiSceneProperties, add_poker_chip, add_table_status, poker_chip_counts,
    table_random_signed,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use std::iter::repeat_n;

pub(in super::super) struct BlackjackWagerSpec {
    pub wager: u32,
    pub net: Option<i32>,
    pub settled: bool,
    pub width: f32,
    pub height: f32,
    pub stack: Entity,
    pub house: Entity,
}

#[derive(Component, Clone, Copy, Default, PartialEq)]
struct WagerStamp {
    settled: bool,
    seed: u64,
}

#[derive(Component, Clone, Copy, Default, PartialEq)]
struct WagerChip(u16);

pub(in super::super) fn register_wager_properties(properties: &mut UiSceneProperties) {
    properties.preserve::<WagerStamp>();
    properties.preserve::<WagerChip>();
}

pub(in super::super) fn add_blackjack_wager(
    commands: &mut UiComposer,
    parent: Entity,
    spec: BlackjackWagerSpec,
    assets: &UiAssets,
) {
    let row = commands.keyed_child(
        parent,
        "wager",
        (Node {
            width: px(spec.width - 8.0),
            height: px(spec.height),
            position_type: PositionType::Relative,
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },),
    );
    let previous = commands.declared::<WagerStamp>(row).copied();
    let seed = previous.map_or_else(|| fastrand::u64(..), |stamp| stamp.seed);
    let scatter = PokerChipScatter {
        seed,
        size: 30.0,
        bounds: Vec2::new(spec.width - 8.0, spec.height),
    };
    if spec.settled {
        let net = spec.net.unwrap_or(0);
        add_table_status(
            commands,
            row,
            TableStatusSpec {
                text: &format!("{net:+}"),
                font_size: 18.0,
                color: if net > 0 {
                    READY
                } else if net < 0 {
                    DANGER
                } else {
                    MUTED
                },
                effect: TableStatusEffect::Bounce,
                elapsed: None,
            },
            assets,
        );
        if previous.is_some_and(|stamp| !stamp.settled) {
            let returned = (i64::from(spec.wager) + i64::from(net)) as u32;
            let (source, target, amount) = if returned == 0 {
                (row, spec.house, spec.wager)
            } else if net == 0 {
                (row, spec.stack, returned)
            } else {
                (spec.house, spec.stack, returned)
            };
            let flight = commands.effect_child(
                target,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        top: percent(50),
                        width: px(90),
                        height: px(36),
                        margin: UiRect::left(px(-45)),
                        ..default()
                    },
                    UiTransform::IDENTITY,
                    GlobalZIndex(1100),
                    Pickable::IGNORE,
                    TablePieceDeal::from_anchor(source, 0.0).transient(SETTLEMENT_CHIP_DURATION),
                ),
            );
            paint_wager(
                commands,
                flight,
                amount,
                &PokerChipScatter {
                    bounds: Vec2::new(90.0, 36.0),
                    ..scatter
                },
                None,
                assets,
            );
        }
    } else {
        paint_wager(
            commands,
            row,
            spec.wager,
            &scatter,
            Some(spec.stack),
            assets,
        );
    }
    commands.entity(row).insert(WagerStamp {
        settled: spec.settled,
        seed,
    });
}

fn paint_wager(
    commands: &mut UiComposer,
    parent: Entity,
    amount: u32,
    scatter: &PokerChipScatter,
    source: Option<Entity>,
    assets: &UiAssets,
) {
    let chips = poker_chip_counts(amount)
        .into_iter()
        .flat_map(|(value, count)| repeat_n(value, count.min(6)))
        .take(12);
    for (index, denomination) in chips.enumerate() {
        let chip = scatter.chip(index, denomination);
        let rotation = chip.rotation;
        let entity = add_poker_chip(commands, parent, chip, assets);
        if let Some(source) = source {
            let stamp = WagerChip(denomination);
            let previous = commands.declared::<WagerChip>(entity).copied();
            let flight = TablePieceDeal::from_anchor(source, index as f32 * 0.045)
                .with_duration(0.7)
                .with_rotation(
                    rotation + table_random_signed(scatter.seed, index as u64 + 91) * 0.7,
                    rotation,
                );
            if previous == Some(stamp) {
                commands
                    .entity(entity)
                    .entry::<TablePieceDeal>()
                    .or_insert(flight);
            } else {
                commands.entity(entity).insert(flight);
            }
            commands.entity(entity).insert(stamp);
        }
    }
}
