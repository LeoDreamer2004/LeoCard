use super::super::add_text;
use crate::app::presentation::{RuleHelp, TEXT, UiComposer};
use crate::app::runtime::UiAssets;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;

pub(crate) fn add_rule_help(
    commands: &mut UiComposer,
    parent: Entity,
    help: &str,
    assets: &UiAssets,
) {
    let question = commands.child(
        parent,
        (
            Button,
            Node {
                width: px(26),
                height: px(26),
                position_type: PositionType::Relative,
                ..default()
            },
            ImageNode::new(assets.home.help_question.clone()),
        ),
    );

    let mut tooltip_image =
        ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    tooltip_image.visual_box = VisualBox::BorderBox;
    let tooltip = commands.child(
        question,
        (
            Node {
                position_type: PositionType::Absolute,
                right: px(30),
                top: px(-7),
                width: px(290),
                padding: UiRect::all(px(15)),
                ..default()
            },
            tooltip_image,
        ),
    );
    commands
        .entity(tooltip)
        .insert((Visibility::Hidden, GlobalZIndex(1000)));
    add_text(commands, tooltip, help, 12.0, TEXT, assets);
    commands.entity(question).insert(RuleHelp { tooltip });
}

pub(crate) fn update_rule_help_tooltips(
    helps: Query<(&PickingInteraction, &RuleHelp), Changed<PickingInteraction>>,
    mut tooltips: Query<&mut Visibility>,
) {
    for (interaction, help) in &helps {
        if let Ok(mut visibility) = tooltips.get_mut(help.tooltip) {
            *visibility = if matches!(
                interaction,
                PickingInteraction::Hovered | PickingInteraction::Pressed
            ) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
