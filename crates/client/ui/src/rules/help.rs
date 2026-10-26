use crate::{UiComposer, add_label};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

#[derive(Component)]
pub struct HelpTooltip {
    pub tooltip: Entity,
}

#[derive(Clone)]
pub struct HelpTooltipStyle<'a> {
    pub font: &'a Handle<Font>,
    pub icon: ImageNode,
    pub background: ImageNode,
    pub text: Color,
    pub width: f32,
}

pub fn add_help_tooltip(
    commands: &mut UiComposer,
    parent: Entity,
    help: &str,
    style: HelpTooltipStyle,
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
            style.icon,
        ),
    );

    let tooltip = commands.child(
        question,
        (
            Node {
                position_type: PositionType::Absolute,
                right: px(30),
                top: px(-7),
                width: px(style.width),
                padding: UiRect::all(px(15)),
                ..default()
            },
            style.background,
        ),
    );
    commands
        .entity(tooltip)
        .insert((Visibility::Hidden, GlobalZIndex(1000)));
    add_label(commands, tooltip, help, 12.0, style.text, style.font);
    commands.entity(question).insert(HelpTooltip { tooltip });
}

pub fn update_help_tooltips(
    helps: Query<(&PickingInteraction, &HelpTooltip), Changed<PickingInteraction>>,
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
