use super::{ScrollContentHeight, Scrollbar, ScrollbarDrag, ScrollbarSkin};
use crate::app::presentation::TransitionOpacity;
use bevy::ecs::query::QueryData;
use bevy::{prelude::*, ui::RelativeCursorPosition};

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct ScrollbarThumb {
    node: &'static mut Node,
    background: Option<&'static mut BackgroundColor>,
    image: Option<&'static mut ImageNode>,
    visibility: &'static mut Visibility,
    transition: &'static TransitionOpacity,
}

pub(crate) fn drag_scrollbar(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<ScrollbarDrag>,
    tracks: Query<(Entity, &Scrollbar, &RelativeCursorPosition, &ComputedNode)>,
    mut scrolls: Query<(
        &ComputedNode,
        &mut ScrollPosition,
        Option<&ScrollContentHeight>,
    )>,
) {
    if !mouse.pressed(MouseButton::Left) {
        drag.active = None;
        return;
    }
    if drag
        .active
        .is_some_and(|(entity, _)| tracks.get(entity).is_err())
    {
        drag.active = None;
    }
    for (entity, scrollbar, cursor, track) in &tracks {
        let Some(normalized) = cursor.normalized else {
            continue;
        };
        let Ok((node, mut position, content_height)) = scrolls.get_mut(scrollbar.scroll) else {
            continue;
        };
        let geometry = scrollbar.geometry(node, &position, track, content_height);
        if geometry.travel == 0.0 || geometry.maximum == 0.0 {
            continue;
        }
        let y = (normalized.y + 0.5) * track.size().y * track.inverse_scale_factor();
        if mouse.just_pressed(MouseButton::Left) && cursor.cursor_over() {
            let grab = if (geometry.offset..=geometry.offset + geometry.thumb).contains(&y) {
                y - geometry.offset
            } else {
                geometry.thumb / 2.0
            };
            drag.active = Some((entity, grab));
        }
        if let Some((active, grab)) = drag.active
            && active == entity
        {
            position.y =
                ((y - grab - geometry.start) / geometry.travel).clamp(0.0, 1.0) * geometry.maximum;
        }
    }
}

pub(crate) fn update_scrollbar(
    drag: Res<ScrollbarDrag>,
    tracks: Query<(Entity, &Scrollbar, &ComputedNode, &RelativeCursorPosition)>,
    scrolls: Query<(&ComputedNode, &ScrollPosition, Option<&ScrollContentHeight>)>,
    mut thumbs: Query<ScrollbarThumb>,
) {
    for (entity, scrollbar, track, cursor) in &tracks {
        let Ok((scroll, position, content_height)) = scrolls.get(scrollbar.scroll) else {
            continue;
        };
        let Ok(mut thumb) = thumbs.get_mut(scrollbar.thumb) else {
            continue;
        };
        let geometry = scrollbar.geometry(scroll, position, track, content_height);
        let over_thumb = cursor.cursor_over()
            && cursor.normalized.is_some_and(|point| {
                let point =
                    (point + Vec2::splat(0.5)) * track.size() * track.inverse_scale_factor();
                let inset = if matches!(scrollbar.skin, ScrollbarSkin::Textured(_)) {
                    2.0
                } else {
                    0.0
                };
                point.x >= inset
                    && point.x <= track.size().x * track.inverse_scale_factor() - inset
                    && (geometry.offset..=geometry.offset + geometry.thumb).contains(&point.y)
            });
        let highlighted = over_thumb || drag.active.is_some_and(|(active, _)| active == entity);
        thumb.node.top = px(geometry.offset);
        thumb.node.height = px(geometry.thumb);
        match &scrollbar.skin {
            ScrollbarSkin::Colored(tint) => {
                *thumb.visibility = if geometry.maximum > 0.0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if let Some(mut color) = thumb.background {
                    let alpha = if highlighted { 0.95 } else { 0.7 };
                    color.0 = tint.with_alpha(alpha * thumb.transition.factor());
                }
            }
            ScrollbarSkin::Textured(images) => {
                if let Some(mut image) = thumb.image {
                    let target = if geometry.maximum <= 0.0 {
                        &images.disabled_thumb
                    } else if highlighted {
                        &images.highlighted_thumb
                    } else {
                        &images.thumb
                    };
                    if image.image != target.image {
                        *image = target.clone();
                    }
                    image
                        .color
                        .set_alpha(target.color.alpha() * thumb.transition.factor());
                }
            }
        }
    }
}
