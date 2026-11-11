use super::entrance::mount_game_button;
use crate::app::presentation::{MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{TextureButton, UiComposer, sliced_image};

#[derive(Clone, Copy)]
pub(crate) enum FramedButtonTone {
    Play,
    Pass,
    Hint,
}

impl FramedButtonTone {
    fn textures(self, assets: &UiAssets) -> (&Handle<Image>, &Handle<Image>) {
        let controls = &assets.controls;
        match self {
            Self::Play => (&controls.game_play_button, &controls.game_play_button_hover),
            Self::Pass => (&controls.game_pass_button, &controls.game_pass_button_hover),
            Self::Hint => (&controls.game_hint_button, &controls.game_hint_button_hover),
        }
    }
}

pub(crate) struct FramedButton<'a> {
    pub label: &'a str,
    pub action: Option<UiAction>,
    pub tone: FramedButtonTone,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub corner_scale: f32,
}

impl<'a> FramedButton<'a> {
    pub(crate) fn scaled(mut self, scale: f32) -> Self {
        self.width *= scale;
        self.height *= scale;
        self.font_size *= scale;
        self.corner_scale *= scale;
        self
    }

    pub(crate) fn action(label: &'a str, action: Option<UiAction>, tone: FramedButtonTone) -> Self {
        Self {
            label,
            action,
            tone,
            width: 164.0,
            height: 54.0,
            font_size: 18.0,
            corner_scale: 0.42,
        }
    }

    #[track_caller]
    pub(crate) fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        assets: &UiAssets,
    ) -> (Entity, Entity) {
        let (normal, hovered) = self.tone.textures(assets);
        let image =
            |texture: &Handle<Image>| sliced_image(texture.clone(), 30.0, self.corner_scale);
        let enabled = self.action.is_some();
        let skin = TextureButton {
            node: Node {
                min_width: px(self.width),
                height: px(self.height),
                padding: UiRect::horizontal(px(16.0 * self.corner_scale / 0.42)),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            normal: image(normal).with_color(Color::WHITE.with_alpha(if enabled {
                1.0
            } else {
                0.55
            })),
            hovered: image(hovered),
            action: self.action,
            label: Some(self.label),
            font_size: self.font_size,
            color: if enabled { TEXT } else { MUTED },
            icon: None,
        };
        let (button, label) = mount_game_button(skin, commands, parent, &assets.font);
        commands.entity(label).insert((
            TextLayout::no_wrap().with_justify(Justify::Center),
            Node {
                flex_shrink: 0.0,
                ..default()
            },
        ));
        (button, label)
    }
}
