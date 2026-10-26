use crate::app::presentation::{ACCENT, HEADER_BG, TEXT, add_text};
use crate::app::runtime::{POKER_CHIP_DENOMINATIONS, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) const CHIP_BET_DURATION: f32 = 0.42;
pub(crate) const CHIP_BET_INTERVAL: f32 = 0.045;

pub(crate) struct PokerChipVisual {
    pub denomination: u16,
    pub position: Vec2,
    pub size: f32,
    pub rotation: f32,
    pub layer: i32,
}

pub(crate) fn add_poker_chip(
    commands: &mut UiComposer,
    parent: Entity,
    chip: PokerChipVisual,
    assets: &UiAssets,
) -> Entity {
    let entity = commands.child(
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(chip.position.x),
                top: px(chip.position.y),
                width: px(chip.size),
                height: px(chip.size),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.poker_chips.image(chip.denomination)),
            UiTransform::from_rotation(Rot2::radians(chip.rotation)),
            ZIndex(chip.layer),
            Pickable::IGNORE,
        ),
    );
    let text = add_text(
        commands,
        entity,
        chip.denomination.to_string(),
        chip.size * 9.0 / 34.0,
        if chip.denomination == 1 {
            HEADER_BG
        } else {
            TEXT
        },
        assets,
    );
    commands.entity(text).insert((
        Pickable::IGNORE,
        TextShadow {
            offset: Vec2::new(0.7, 0.8),
            color: Color::BLACK.with_alpha(0.55),
        },
    ));
    entity
}

pub(crate) fn add_horizontal_chip_group(
    commands: &mut UiComposer,
    parent: Entity,
    denomination: u16,
    count: usize,
    assets: &UiAssets,
) {
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(34.0 + count.saturating_sub(1) as f32 * 11.0),
            height: px(34),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    for index in 0..count {
        add_poker_chip(
            commands,
            group,
            PokerChipVisual {
                denomination,
                position: Vec2::new(index as f32 * 11.0, 0.0),
                size: 34.0,
                rotation: 0.0,
                layer: index as i32,
            },
            assets,
        );
    }
}

pub(crate) fn add_poker_chip_value(
    commands: &mut UiComposer,
    parent: Entity,
    value: u32,
    assets: &UiAssets,
) {
    let fan = spawn_node(
        commands,
        parent,
        Node {
            width: px(35),
            height: px(30),
            position_type: PositionType::Relative,
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    for (denomination, x, y, rotation, layer) in [(1, 1.0, 6.0, -0.22, 0), (5, 12.0, 1.0, 0.16, 1)]
    {
        add_poker_chip(
            commands,
            fan,
            PokerChipVisual {
                denomination,
                position: Vec2::new(x, y),
                size: 24.0,
                rotation,
                layer,
            },
            assets,
        );
    }
    let digits = value.to_string();
    let font_size = (28.0 - digits.len().saturating_sub(3) as f32 * 2.5).max(17.0);
    let value = add_text(commands, parent, digits, font_size, ACCENT, assets);
    commands.entity(value).insert((
        TextLayout::no_wrap(),
        TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        },
        Pickable::IGNORE,
    ));
}

pub(crate) fn poker_chip_counts(mut value: u32) -> Vec<(u16, usize)> {
    POKER_CHIP_DENOMINATIONS
        .into_iter()
        .filter_map(|denomination| {
            let count = value / u32::from(denomination);
            value %= u32::from(denomination);
            (count > 0).then_some((denomination, count as usize))
        })
        .collect()
}
