use crate::UiComposer;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::FontSourceTemplate;
use std::f32::consts;

pub struct TextOutline {
    pub color: Color,
    pub width: f32,
}

pub struct TextOutlineLayers {
    pub fill: Entity,
    pub strokes: [Entity; 8],
}

impl TextOutline {
    /// The foreground provides the natural size; outline layers never affect layout.
    #[track_caller]
    pub fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        text: &str,
        font: TextFont,
        color: Color,
    ) -> TextOutlineLayers {
        let diagonal = consts::FRAC_1_SQRT_2;
        let strokes = [
            Vec2::new(-diagonal, -diagonal),
            Vec2::NEG_Y,
            Vec2::new(diagonal, -diagonal),
            Vec2::X,
            Vec2::splat(diagonal),
            Vec2::Y,
            Vec2::new(-diagonal, diagonal),
            Vec2::NEG_X,
        ]
        .map(|offset| {
            commands.child(
                parent,
                (
                    Text::new(text),
                    font.clone(),
                    TextLayout::no_wrap().with_justify(Justify::Center),
                    TextColor(self.color),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        ..default()
                    },
                    UiTransform::from_translation(Val2::px(
                        offset.x * self.width,
                        offset.y * self.width,
                    )),
                    Pickable::IGNORE,
                ),
            )
        });
        let fill = commands.child(
            parent,
            (
                Text::new(text),
                font,
                TextLayout::no_wrap().with_justify(Justify::Center),
                TextColor(color),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
        TextOutlineLayers { fill, strokes }
    }
}

#[track_caller]
pub fn add_label(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    font: &Handle<Font>,
) -> Entity {
    commands.node(parent, label_scene(font.clone(), text, size, color))
}

pub fn label_scene(
    font: Handle<Font>,
    text: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Scene {
    let text = Text::new(text);
    let font_source = FontSourceTemplate::Handle(font.into());
    bsn! {
        text
        TextFont { font: font_source, font_size: px(size) }
        TextColor(color)
    }
}
