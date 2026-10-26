use crate::UiComposer;
use bevy::prelude::*;
use bevy::text::FontSourceTemplate;

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
