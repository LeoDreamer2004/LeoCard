use bevy::prelude::*;

/// Applications may retarget a live bubble after replacing its anchor entity.
#[derive(Component)]
pub struct AnchoredBubble {
    pub anchor: Option<Entity>,
    pub(super) anchor_offset: Vec2,
    pub(super) text: Option<Entity>,
    pub(super) image: Option<Entity>,
    pub(super) size: Vec2,
    pub(super) elapsed: f32,
    pub(super) duration: f32,
    pub(super) missing_secs: f32,
    pub(super) background_color: Color,
    pub(super) text_color: Color,
}

pub enum BubbleContent<'a> {
    Text(&'a str),
    Image(Handle<Image>),
}

pub struct BubbleStyle {
    pub background: ImageNode,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub text_color: Color,
    pub max_width: f32,
    pub anchor_offset: Vec2,
    pub z_index: i32,
}

impl BubbleStyle {
    pub fn new(background: ImageNode, font: Handle<Font>, text_color: Color) -> Self {
        Self {
            background,
            font,
            font_size: 13.0,
            text_color,
            max_width: 230.0,
            anchor_offset: Vec2::ZERO,
            z_index: 0,
        }
    }
}
