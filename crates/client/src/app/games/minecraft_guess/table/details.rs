use super::super::MinecraftGuessAssets;
use crate::app::presentation::{AnchoredPopup, TEXT, UiComposer, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_minecraft_guess::Attribute;

#[derive(Component)]
pub(in super::super) struct AttributeHint(Entity);

#[derive(Component)]
pub(in super::super) struct AttributePopup;

impl AttributeHint {
    pub fn preview(attribute: &Attribute) -> String {
        let values = attribute.display_values();
        if values.len() > 4 {
            format!("{}、{}等 {} 项", values[0], values[1], values.len())
        } else {
            values.join("、")
        }
    }

    pub fn spawn(
        commands: &mut UiComposer,
        layer: Entity,
        anchor: Entity,
        attribute: &Attribute,
        assets: &UiAssets,
        game_assets: &MinecraftGuessAssets,
    ) {
        if attribute.display_values().len() <= 4 {
            return;
        }
        let popup = spawn_node(
            commands,
            layer,
            Node {
                position_type: PositionType::Absolute,
                width: px(440),
                max_width: percent(95),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        commands.entity(popup).insert((
            game_assets.popup_frame(),
            AttributePopup,
            AnchoredPopup(anchor),
            GlobalZIndex(1600),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
        let heading = format!("{}（{} 项）", attribute.label, attribute.values.len());
        let heading = add_text(commands, popup, heading, 16.0, TEXT, assets);
        commands.entity(heading).insert(Pickable::IGNORE);
        let text = add_text(commands, popup, attribute.text(), 14.0, TEXT, assets);
        commands.entity(text).insert((
            Pickable::IGNORE,
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
        ));
        commands.entity(anchor).insert((
            AttributeHint(popup),
            Pickable::default(),
            PickingInteraction::default(),
        ));
    }
}

pub(in super::super) fn sync_attribute_hints(
    hints: Query<(&PickingInteraction, &AttributeHint, &InheritedVisibility)>,
    mut popups: Query<(Entity, &mut Visibility), With<AttributePopup>>,
) {
    let hovered = hints.iter().find_map(|(interaction, hint, visibility)| {
        (visibility.get() && *interaction != PickingInteraction::None).then_some(hint.0)
    });
    for (entity, mut visibility) in &mut popups {
        visibility.set_if_neq(if hovered == Some(entity) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}
