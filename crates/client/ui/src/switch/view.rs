use super::state::{SwitchAnimation, SwitchState};
use crate::{CustomButtonMotion, TransitionOpacity, UiComposer};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

/// Images, colors and scale are supplied by the caller's visual theme.
pub struct SwitchStyle {
    pub track: ImageNode,
    pub thumb: ImageNode,
    pub off_color: Color,
    pub on_color: Color,
    pub disabled_mask: Color,
    pub scale: f32,
}

pub fn add_switch<A: Component + Clone>(
    commands: &mut UiComposer,
    parent: Entity,
    selected: bool,
    action: Option<A>,
    mut style: SwitchStyle,
) -> Entity {
    let enabled = action.is_some();
    let progress = f32::from(selected);
    let scale = style.scale;
    // Keep authored colors stable so recomposition does not reset an active transition.
    style.track.color = style.off_color;
    let switch = commands.child(
        parent,
        (
            CustomButtonMotion,
            TransitionOpacity::default(),
            if enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            },
            Node {
                width: px(56.0 * scale),
                height: px(30.0 * scale),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(5.0 * scale)),
                overflow: Overflow::clip(),
                ..default()
            },
            style.track,
        ),
    );
    if let Some(action) = action {
        commands.entity(switch).insert((Button, action));
    }
    commands.child(
        switch,
        (
            SwitchAnimation {
                owner: switch,
                progress,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(3.0 * scale),
                top: px(3.0 * scale),
                width: px(24.0 * scale),
                height: px(24.0 * scale),
                ..default()
            },
            style.thumb,
            ZIndex(1),
            Pickable::IGNORE,
        ),
    );
    let mask = commands.child(
        switch,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                border_radius: BorderRadius::all(px(5.0 * scale)),
                ..default()
            },
            BackgroundColor(style.disabled_mask),
            if enabled {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
            ZIndex(2),
            Pickable::IGNORE,
        ),
    );
    commands.entity(switch).insert(SwitchState {
        selected,
        enabled,
        scale,
        off_color: style.off_color,
        on_color: style.on_color,
        mask,
        mask_color: style.disabled_mask,
    });
    switch
}
