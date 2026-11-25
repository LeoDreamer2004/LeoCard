use crate::{UiComposer, add_label};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::OverrideClip;
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
) -> Entity {
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
            style.icon.clone(),
        ),
    );

    attach_help_tooltip(commands, question, help, style)
}

pub fn attach_help_tooltip(
    commands: &mut UiComposer,
    target: Entity,
    help: &str,
    style: HelpTooltipStyle,
) -> Entity {
    let tooltip = commands.child(
        target,
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
        .insert((Visibility::Hidden, GlobalZIndex(1000), OverrideClip));
    add_label(commands, tooltip, help, 12.0, style.text, style.font);
    commands.entity(target).insert(HelpTooltip { tooltip });
    tooltip
}

pub fn update_help_tooltips(
    helps: Query<(&PickingInteraction, &HelpTooltip)>,
    mut tooltips: Query<&mut Visibility>,
) {
    for (interaction, help) in &helps {
        if let Ok(mut visibility) = tooltips.get_mut(help.tooltip) {
            let desired = if matches!(
                interaction,
                PickingInteraction::Hovered | PickingInteraction::Pressed
            ) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *visibility != desired {
                *visibility = desired;
            }
        }
    }
}
