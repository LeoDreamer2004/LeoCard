use super::state::{SwitchAnimation, SwitchState};
use crate::TransitionOpacity;
use bevy::color::Mix;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

pub fn animate_switches(
    time: Res<Time>,
    mut switches: Query<
        (
            &SwitchState,
            &TransitionOpacity,
            Option<&PickingInteraction>,
            &mut ImageNode,
        ),
        Without<SwitchAnimation>,
    >,
    mut thumbs: Query<(&mut SwitchAnimation, &mut Node, &mut ImageNode)>,
    mut masks: Query<&mut BackgroundColor>,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (mut animation, mut node, mut image) in &mut thumbs {
        let Ok((switch, transition, interaction, mut track)) = switches.get_mut(animation.owner)
        else {
            continue;
        };
        animation.progress += (f32::from(switch.selected) - animation.progress) * response;
        node.left = px((3.0 + 26.0 * animation.progress) * switch.scale);
        let opacity = transition.factor();
        let color = switch.off_color.mix(&switch.on_color, animation.progress);
        track.color = color.with_alpha(color.alpha() * opacity);
        if let Ok(mut mask) = masks.get_mut(switch.mask) {
            mask.0 = switch
                .mask_color
                .with_alpha(switch.mask_color.alpha() * opacity);
        }
        let hovered =
            switch.enabled && interaction.is_some_and(|value| *value != PickingInteraction::None);
        let brightness = if hovered { 1.0 } else { 0.88 };
        image.color = Color::srgba(brightness, brightness, brightness, opacity);
    }
}
