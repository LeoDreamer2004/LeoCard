use super::add_horizontal_chip_group;
use crate::app::presentation::{ACCENT, MUTED, add_text, card_background_image};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_poker_balance_panel(
    commands: &mut UiComposer,
    parent: Entity,
    portrait_size: Vec2,
    balance: u32,
    counts: &[(u16, usize)],
    assets: &UiAssets,
) -> Entity {
    let width = if cfg!(target_os = "android") {
        250.0
    } else {
        300.0
    };
    let panel = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(portrait_size.x + 8.0),
            top: px((portrait_size.y - 66.0) * 0.5),
            width: px(width),
            min_height: px(66),
            padding: UiRect::new(px(76), px(6), px(6), px(6)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert((
        card_background_image(assets.home.game_card.clone(), 1.0),
        GlobalZIndex(1000),
        Pickable::IGNORE,
    ));
    let value_area = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(6),
            top: px(4),
            bottom: px(4),
            width: px(65),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(-2),
            ..default()
        },
        None,
    );
    commands.entity(value_area).insert(Pickable::IGNORE);
    add_text(commands, value_area, "剩余", 10.0, MUTED, assets);
    let digits = balance.to_string();
    let font_size = (28.0 - digits.len().saturating_sub(3) as f32 * 2.5).max(17.0);
    let value = add_text(commands, value_area, digits, font_size, ACCENT, assets);
    commands.entity(value).insert(TextShadow {
        offset: Vec2::new(1.5, 2.0),
        color: Color::BLACK.with_alpha(0.82),
    });
    let chip_area = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Relative,
            width: percent(100),
            height: px(38),
            ..default()
        },
        None,
    );
    let chip_width = counts
        .iter()
        .map(|(_, count)| 34.0 + count.saturating_sub(1) as f32 * 11.0)
        .sum::<f32>()
        + counts.len().saturating_sub(1) as f32 * 10.0;
    let scale = if cfg!(target_os = "android") {
        ((width - 82.0) / chip_width.max(1.0)).min(1.0)
    } else {
        1.0
    };
    let chips = spawn_node(
        commands,
        chip_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(chip_width),
            height: px(38),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    // Keep the numeric balance full size; fit only the decorative chips from their left edge.
    commands.entity(chips).insert(UiTransform {
        translation: Val2::px((scale - 1.0) * chip_width * 0.5, 0.0),
        scale: Vec2::splat(scale),
        ..default()
    });
    for &(denomination, count) in counts {
        add_horizontal_chip_group(commands, chips, denomination, count, assets);
    }
    panel
}
