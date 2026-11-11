use super::entrance::mount_game_button;
use super::skin::TableButtonSkin;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::text::FontSource;
use leocard_ui::{TextureButton, UiComposer, sliced_image};

pub(crate) struct TableActionButton<'a> {
    pub label: &'a str,
    pub action: Option<UiAction>,
    pub skin: TableButtonSkin,
    pub size: Vec2,
    pub font_size: f32,
}

impl<'a> TableActionButton<'a> {
    pub(crate) fn new(label: &'a str, action: Option<UiAction>, skin: TableButtonSkin) -> Self {
        Self {
            label,
            action,
            skin,
            size: Vec2::new(164.0, 54.0),
            font_size: 25.0,
        }
    }

    pub(crate) fn scaled(mut self, scale: f32) -> Self {
        self.size *= scale;
        self.font_size *= scale;
        self
    }

    #[track_caller]
    pub(crate) fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        assets: &UiAssets,
    ) -> (Entity, Entity) {
        let visual = self.skin.visual(assets);
        let enabled = self.action.is_some();
        let scale = self.size.y / 96.0;
        let image = sliced_image(visual.image.clone(), visual.border, scale);
        let text = visual.text.with_alpha(if enabled { 1.0 } else { 0.78 });
        let skin = TextureButton {
            node: Node {
                min_width: px(self.size.x),
                height: px(self.size.y),
                padding: UiRect::horizontal(px((visual.border.min_inset.x
                    + visual.border.max_inset.x)
                    * scale
                    * 0.5)),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            normal: image
                .clone()
                .with_color(Color::WHITE.with_alpha(if enabled { 1.0 } else { 0.5 })),
            hovered: image.with_color(Color::linear_rgb(1.18, 1.18, 1.18)),
            action: self.action,
            label: Some(self.label),
            font_size: self.font_size,
            color: text,
            icon: None,
        };
        let (button, label) = mount_game_button(skin, commands, parent, visual.font);
        commands.entity(label).insert((
            TextFont {
                font: FontSource::list([visual.font.clone(), assets.font.clone()]),
                weight: visual.weight,
                ..TextFont::from_font_size(self.font_size)
            },
            TextLayout::no_wrap().with_justify(Justify::Center),
            UiTransform::from_translation(Val2::px(0.0, -self.size.y * 0.025)),
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            TextShadow {
                offset: Vec2::new(0.45, 1.0),
                color: Color::BLACK.with_alpha(if enabled { 0.5 } else { 0.25 }),
            },
        ));
        (button, label)
    }
}
