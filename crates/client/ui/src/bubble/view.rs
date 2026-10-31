use super::{AnchoredBubble, BubbleContent, BubbleStyle};
use crate::{UiComposer, add_label};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct Bubble<'a> {
    pub anchor: Entity,
    pub content: BubbleContent<'a>,
    pub duration: f32,
}

impl Bubble<'_> {
    pub fn spawn(self, commands: &mut UiComposer, layer: Entity, style: &BubbleStyle) -> Entity {
        let (width, height, padding) = match &self.content {
            BubbleContent::Text(text) => (
                (text.chars().count() as f32 * style.font_size + 26.0)
                    .clamp(76.0, style.max_width.max(76.0)),
                38.0,
                UiRect::axes(px(11), px(7)),
            ),
            BubbleContent::Image(_) => (86.0, 86.0, UiRect::all(px(7))),
        };
        let mut background = style.background.clone();
        background.color = background.color.with_alpha(0.0);
        let entity = commands.child(
            layer,
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(width),
                    min_width: px(width),
                    max_width: px(width),
                    min_height: px(height),
                    padding,
                    align_items: AlignItems::Center,
                    justify_content: if matches!(&self.content, BubbleContent::Image(_)) {
                        JustifyContent::Center
                    } else {
                        JustifyContent::FlexStart
                    },
                    ..default()
                },
                background,
                UiTransform {
                    translation: Val2::px(0.0, 9.0),
                    scale: Vec2::splat(0.88),
                    ..UiTransform::IDENTITY
                },
                Visibility::Hidden,
                GlobalZIndex(style.z_index),
                Pickable::IGNORE,
            ),
        );
        let (text, image) = match self.content {
            BubbleContent::Text(text) => {
                let text = add_label(
                    commands,
                    entity,
                    text,
                    style.font_size,
                    Color::NONE,
                    &style.font,
                );
                commands.entity(text).insert(Pickable::IGNORE);
                (Some(text), None)
            }
            BubbleContent::Image(texture) => {
                let image = commands.child(
                    entity,
                    (
                        Node {
                            width: px(70),
                            height: px(70),
                            ..default()
                        },
                        ImageNode::new(texture).with_color(Color::NONE),
                        Pickable::IGNORE,
                    ),
                );
                (None, Some(image))
            }
        };
        commands.entity(entity).insert(AnchoredBubble {
            anchor: Some(self.anchor),
            anchor_offset: style.anchor_offset,
            text,
            image,
            size: Vec2::new(width, height),
            elapsed: 0.0,
            duration: self.duration,
            missing_secs: 0.0,
            background_color: style.background.color,
            text_color: style.text_color,
        });
        entity
    }
}
