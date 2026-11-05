use super::super::TexasTableLayout;
use super::{TexasRaiseFill, TexasRaiseSlider, raise_fraction};
use crate::app::presentation::SilentButton;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, LinearGradient, RelativeCursorPosition};
use bevy::ui_widgets::Button;
use leocard_texas_holdem::TexasRaiseSizing;
use leocard_ui::{CustomButtonMotion, TouchDragTarget, UiComposer, sliced_image};

pub(super) fn add_raise_slider(
    commands: &mut UiComposer,
    parent: Entity,
    sizing: TexasRaiseSizing,
    target: u32,
    assets: &UiAssets,
) {
    let scale = TexasTableLayout::ACTION_SCALE;
    let track = commands.child(
        parent,
        (
            Button,
            CustomButtonMotion,
            SilentButton,
            TexasRaiseSlider(sizing),
            TouchDragTarget,
            RelativeCursorPosition::default(),
            Node {
                flex_grow: 1.0,
                height: px(38.0 * scale),
                min_width: px(40.0 * scale),
                ..default()
            },
        ),
    );
    let groove = commands.child(
        track,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                height: px(38.0 * scale),
                ..default()
            },
            sliced_image(assets.home.slider.clone(), 32.0, 0.375 * scale),
            Pickable::IGNORE,
        ),
    );
    let interior = commands.child(
        groove,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(3.0 * scale),
                right: px(3.0 * scale),
                // The source texture has transparent vertical margins; the inset follows
                // its visible inner rim at the slicer's 0.375 corner scale.
                top: px(7.5 * scale),
                bottom: px(7.5 * scale),
                border_radius: BorderRadius::all(px(8.0 * scale)),
                overflow: Overflow::clip(),
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
    // The coloured surface fills the recessed track; there is deliberately no thumb.
    let fraction = raise_fraction(sizing, target);
    commands.child(
        interior,
        (
            TexasRaiseFill { sizing, target },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                bottom: px(0),
                width: percent(3.5 + 96.5 * fraction),
                border_radius: BorderRadius::all(px(6.0 * scale)),
                border: UiRect::top(px(scale)),
                ..default()
            },
            raise_surface(fraction),
            BorderColor::all(Color::WHITE.with_alpha(0.32)),
            Pickable::IGNORE,
        ),
    );
}

/// Warm the entire lacquered fill as the selected stake increases.
pub(super) fn raise_surface(fraction: f32) -> BackgroundGradient {
    let hue = 125.0 * (1.0 - fraction.clamp(0.0, 1.0));
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::percent(Color::hsl(hue, 0.47, 0.68), 0.0),
        ColorStop::percent(Color::hsl(hue, 0.56, 0.47), 34.0),
        ColorStop::percent(Color::hsl(hue, 0.62, 0.32), 52.0),
        ColorStop::percent(Color::hsl(hue, 0.54, 0.43), 100.0),
    ]))
}
