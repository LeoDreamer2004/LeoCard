//! Shared visual subtree opacity, independent of page and feature state.

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, Gradient};

#[derive(Component, Clone)]
pub(crate) struct TransitionFadeBase {
    image: Option<Color>,
    text: Option<Color>,
    background: Option<Color>,
    border: Option<BorderColor>,
    gradient: Option<BackgroundGradient>,
}

pub(crate) type TransitionVisuals<'w, 's, Filter = ()> = Query<
    'w,
    's,
    (
        Option<&'static mut ImageNode>,
        Option<&'static mut TextColor>,
        Option<&'static mut BackgroundColor>,
        Option<&'static mut BorderColor>,
        Option<&'static mut BackgroundGradient>,
        Option<&'static TransitionFadeBase>,
    ),
    Filter,
>;

pub(crate) fn fade_panel<Filter: QueryFilter>(
    root: Entity,
    opacity: f32,
    children: &Query<&Children>,
    visuals: &mut TransitionVisuals<Filter>,
    commands: &mut Commands,
) {
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        if let Ok(child_entities) = children.get(entity) {
            pending.extend(child_entities.iter());
        }
        let Ok((mut image, mut text, mut background, mut border, mut gradient, cached)) =
            visuals.get_mut(entity)
        else {
            continue;
        };
        let base = cached.cloned().unwrap_or_else(|| TransitionFadeBase {
            image: image.as_ref().map(|image| image.color),
            text: text.as_ref().map(|text| text.0),
            background: background.as_ref().map(|background| background.0),
            border: border.as_ref().map(|border| **border),
            gradient: gradient.as_ref().map(|gradient| (**gradient).clone()),
        });
        if cached.is_none() {
            commands.entity(entity).insert(base.clone());
        }
        if let (Some(image), Some(color)) = (image.as_mut(), base.image) {
            image.color = color.with_alpha(color.alpha() * opacity);
        }
        if let (Some(text), Some(color)) = (text.as_mut(), base.text) {
            text.0 = color.with_alpha(color.alpha() * opacity);
        }
        if let (Some(background), Some(color)) = (background.as_mut(), base.background) {
            background.0 = color.with_alpha(color.alpha() * opacity);
        }
        if let (Some(border), Some(colors)) = (border.as_mut(), base.border) {
            border.top = colors.top.with_alpha(colors.top.alpha() * opacity);
            border.right = colors.right.with_alpha(colors.right.alpha() * opacity);
            border.bottom = colors.bottom.with_alpha(colors.bottom.alpha() * opacity);
            border.left = colors.left.with_alpha(colors.left.alpha() * opacity);
        }
        if let (Some(gradient), Some(original)) = (gradient.as_mut(), base.gradient) {
            **gradient = original;
            for gradient in &mut gradient.0 {
                match gradient {
                    Gradient::Linear(gradient) => {
                        for stop in &mut gradient.stops {
                            stop.color = stop.color.with_alpha(stop.color.alpha() * opacity);
                        }
                    }
                    Gradient::Radial(gradient) => {
                        for stop in &mut gradient.stops {
                            stop.color = stop.color.with_alpha(stop.color.alpha() * opacity);
                        }
                    }
                    Gradient::Conic(gradient) => {
                        for stop in &mut gradient.stops {
                            stop.color = stop.color.with_alpha(stop.color.alpha() * opacity);
                        }
                    }
                }
            }
        }
    }
}
