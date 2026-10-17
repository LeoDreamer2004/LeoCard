use super::add_horizontal_chip_group;
use crate::app::presentation::{
    ACCENT, MUTED, UiComposer, add_text, card_background_image, spawn_node,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;

pub(crate) fn add_poker_balance_panel(
    commands: &mut UiComposer,
    parent: Entity,
    portrait_size: Vec2,
    balance: u32,
    counts: &[(u16, usize)],
    assets: &UiAssets,
) -> Entity {
    let panel = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(portrait_size.x + 8.0),
            top: px((portrait_size.y - 66.0) * 0.5),
            width: px(300),
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
        GlobalZIndex(1500),
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
    let chips = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(38),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    for &(denomination, count) in counts {
        add_horizontal_chip_group(commands, chips, denomination, count, assets);
    }
    panel
}
