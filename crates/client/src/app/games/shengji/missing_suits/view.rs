use super::{MissingSuitsUi, state::MissingSuitMarker};
use crate::app::presentation::{ACCENT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::SeatSide;
use bevy::{picking::Pickable, prelude::*};
use leocard_protocol::ShengjiPlayerState;
use leocard_shengji::Category;

pub(in super::super) fn render_missing_suits(
    commands: &mut Commands,
    avatar: Entity,
    player: &ShengjiPlayerState,
    side: SeatSide,
    state: &MissingSuitsUi,
    assets: &UiAssets,
) {
    if !state.active {
        return;
    }
    let column = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            left: if matches!(side, SeatSide::Left) {
                px(-50)
            } else {
                Val::Auto
            },
            right: if matches!(side, SeatSide::Left) {
                Val::Auto
            } else {
                px(-50)
            },
            bottom: px(-30.4),
            width: px(30),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    commands
        .entity(column)
        .insert((Pickable::IGNORE, GlobalZIndex(40)));
    let circle = spawn_node(
        commands,
        column,
        Node {
            width: px(24),
            height: px(24),
            border: UiRect::all(px(1.5)),
            border_radius: BorderRadius::all(percent(50)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(circle)
        .insert((BorderColor::all(ACCENT), Pickable::IGNORE));
    add_text(commands, circle, "缺", 14.0, ACCENT, assets);
    for &door in &player.missing_suits {
        let marker = match door {
            Category::Trump => add_text(commands, column, "主", 23.0, ACCENT, assets),
            Category::Suit(suit) => {
                let icon = commands
                    .spawn((
                        Node {
                            width: px(26),
                            height: px(26),
                            ..default()
                        },
                        ImageNode::new(
                            assets.playing_cards.suits[usize::from(suit.bid_strength())].clone(),
                        ),
                    ))
                    .id();
                commands.entity(column).add_child(icon);
                icon
            }
            Category::Mixed => continue,
        };
        commands.entity(marker).insert((
            MissingSuitMarker {
                player: player.id,
                door,
            },
            UiTransform::IDENTITY,
            Pickable::IGNORE,
        ));
    }
}
