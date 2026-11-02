use super::super::ACCENT;
use super::add_title;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::FontWeight;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_player_role_badge_row(commands: &mut UiComposer, avatar: Entity) -> Entity {
    let row = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            right: px(-11),
            bottom: px(-9),
            flex_direction: FlexDirection::Row,
            column_gap: px(3),
            ..default()
        },
        None,
    );
    commands.entity(row).insert((ZIndex(100), Pickable::IGNORE));
    row
}

pub(crate) fn add_player_role_badge(
    commands: &mut UiComposer,
    row: Entity,
    label: &str,
    assets: &UiAssets,
) -> Entity {
    let badge = spawn_node(
        commands,
        row,
        Node {
            width: px(22),
            min_width: px(22),
            height: px(22),
            flex_shrink: 0.0,
            border: UiRect::all(px(1.25)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(ACCENT),
    );
    commands.entity(badge).insert((
        BorderColor::all(Color::srgb(0.025, 0.020, 0.015)),
        BoxShadow::new(Color::BLACK.with_alpha(0.44), px(0), px(2), px(0.5), px(4)),
        Pickable::IGNORE,
    ));
    let text = add_title(
        commands,
        badge,
        label,
        if label.chars().count() == 1 {
            12.0
        } else {
            9.5
        },
        Color::srgb(0.025, 0.020, 0.015),
        assets,
    );
    commands
        .entity(text)
        .insert((TextLayout::no_wrap(), Pickable::IGNORE));
    commands
        .entity(text)
        .entry::<TextFont>()
        .and_modify(|mut font| {
            font.weight = FontWeight::BOLD;
        });
    badge
}
