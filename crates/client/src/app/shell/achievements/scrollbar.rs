use crate::app::presentation::spawn_node;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::state::ACCENT;

#[derive(Component)]
pub(super) struct AchievementScrollbar {
    scroll: Entity,
    thumb: Entity,
}

#[derive(Resource, Default)]
pub(super) struct AchievementScrollbarDrag {
    active: Option<(Entity, f32)>,
}

struct ScrollGeometry {
    maximum: f32,
    thumb: f32,
    travel: f32,
    offset: f32,
}

impl ScrollGeometry {
    fn new(scroll: &ComputedNode, position: &ScrollPosition, track: &ComputedNode) -> Self {
        let height = track.size().y * track.inverse_scale_factor();
        let content = scroll.content_size().y * scroll.inverse_scale_factor();
        let visible = scroll.size().y * scroll.inverse_scale_factor();
        let maximum = (content - visible).max(0.0);
        let thumb = (height * (visible / content.max(1.0)).min(1.0))
            .max(32.0)
            .min(height);
        let travel = (height - thumb).max(0.0);
        let offset = if maximum > 0.0 {
            travel * position.y.clamp(0.0, maximum) / maximum
        } else {
            0.0
        };
        Self {
            maximum,
            thumb,
            travel,
            offset,
        }
    }
}

impl AchievementScrollbar {
    pub(super) fn spawn(commands: &mut Commands, parent: Entity, scroll: Entity) {
        let track = spawn_node(
            commands,
            parent,
            Node {
                width: px(12),
                height: percent(100),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            Some(ACCENT.with_alpha(0.12)),
        );
        let thumb = spawn_node(
            commands,
            track,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            Some(ACCENT.with_alpha(0.7)),
        );
        commands
            .entity(track)
            .insert((Self { scroll, thumb }, RelativeCursorPosition::default()));
    }
}

pub(super) fn drag_scrollbar(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<AchievementScrollbarDrag>,
    tracks: Query<(
        Entity,
        &AchievementScrollbar,
        &RelativeCursorPosition,
        &ComputedNode,
    )>,
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
        let geometry = ScrollGeometry::new(node, &position, track);
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
            position.y = ((y - grab) / geometry.travel).clamp(0.0, 1.0) * geometry.maximum;
        }
    }
}

pub(super) fn update_scrollbar(
    tracks: Query<(
        &AchievementScrollbar,
        &ComputedNode,
        &RelativeCursorPosition,
    )>,
    scrolls: Query<(&ComputedNode, &ScrollPosition)>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor, &mut Visibility)>,
) {
    for (scrollbar, track, cursor) in &tracks {
        let Ok((scroll, position)) = scrolls.get(scrollbar.scroll) else {
            continue;
        };
        let Ok((mut node, mut color, mut visibility)) = thumbs.get_mut(scrollbar.thumb) else {
            continue;
        };
        let geometry = ScrollGeometry::new(scroll, position, track);
        node.top = px(geometry.offset);
        node.height = px(geometry.thumb);
        *visibility = if geometry.maximum > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        color.0 = ACCENT.with_alpha(if cursor.cursor_over() { 0.95 } else { 0.7 });
    }
}
