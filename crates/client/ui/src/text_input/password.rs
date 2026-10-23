//! Paint masked dots at native glyph positions, preserving caret and horizontal scrolling.
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::{EditableText, TextLayoutInfo};

#[derive(Component)]
pub(super) struct PasswordMask {
    pub editor: Entity,
    pub font_size: f32,
    pub color: Color,
}

pub(super) fn sync_password_masks(
    mut commands: Commands,
    masks: Query<(Entity, &PasswordMask, Option<&Children>)>,
    editors: Query<(&EditableText, &TextLayoutInfo)>,
    mut dots: Query<&mut Node, Without<PasswordMask>>,
) {
    for (entity, mask, children) in &masks {
        let Ok((editor, layout)) = editors.get(mask.editor) else {
            continue;
        };
        let scale = layout.scale_factor.recip();
        let size = mask.font_size * 0.24;
        let existing: Vec<_> = children.into_iter().flatten().copied().collect();
        for (index, glyph) in layout.glyphs.iter().enumerate() {
            let node = Node {
                position_type: PositionType::Absolute,
                left: px(glyph.position.x * scale - editor.viewport.offset.x - size * 0.5),
                top: px((mask.font_size * 1.5 - size) * 0.5),
                width: px(size),
                height: px(size),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            };
            if let Some(&child) = existing.get(index) {
                if let Ok(mut current) = dots.get_mut(child)
                    && *current != node
                {
                    *current = node;
                }
            } else {
                commands.spawn((
                    node,
                    BackgroundColor(mask.color),
                    Pickable::IGNORE,
                    ChildOf(entity),
                ));
            }
        }
        for &child in existing.iter().skip(layout.glyphs.len()) {
            commands.entity(child).despawn();
        }
    }
}
