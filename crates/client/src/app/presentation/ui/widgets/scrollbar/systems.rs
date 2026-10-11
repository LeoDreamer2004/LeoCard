use super::{Scrollbar, ScrollbarDrag, ScrollbarSkin};
use bevy::{prelude::*, ui::RelativeCursorPosition};

pub(crate) fn drag_scrollbar(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<ScrollbarDrag>,
    tracks: Query<(Entity, &Scrollbar, &RelativeCursorPosition, &ComputedNode)>,
    mut scrolls: Query<(&ComputedNode, &mut ScrollPosition)>,
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
        let Ok((node, mut position)) = scrolls.get_mut(scrollbar.scroll) else {
            continue;
        };
        let geometry = scrollbar.geometry(node, &position, track);
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
    tracks: Query<(&Scrollbar, &ComputedNode, &RelativeCursorPosition)>,
    scrolls: Query<(&ComputedNode, &ScrollPosition)>,
    mut thumbs: Query<(
        &mut Node,
        Option<&mut BackgroundColor>,
        Option<&mut ImageNode>,
        &mut Visibility,
    )>,
) {
    for (scrollbar, track, cursor) in &tracks {
        let Ok((scroll, position)) = scrolls.get(scrollbar.scroll) else {
            continue;
        };
        let Ok((mut node, color, image, mut visibility)) = thumbs.get_mut(scrollbar.thumb) else {
            continue;
        };
        let geometry = scrollbar.geometry(scroll, position, track);
        node.top = px(geometry.offset);
        node.height = px(geometry.thumb);
        match &scrollbar.skin {
            ScrollbarSkin::Colored(tint) => {
                *visibility = if geometry.maximum > 0.0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if let Some(mut color) = color {
                    color.0 = tint.with_alpha(if cursor.cursor_over() { 0.95 } else { 0.7 });
                }
            }
            ScrollbarSkin::Textured(images) => {
                if let Some(mut image) = image {
                    let target = if geometry.maximum > 0.0 {
                        &images.thumb
                    } else {
                        &images.disabled_thumb
                    };
                    if image.image != target.image {
                        *image = target.clone();
                    }
                }
            }
        }
    }
}
