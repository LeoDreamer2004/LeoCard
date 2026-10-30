//! 牌桌操作按钮共用的纹理、悬停层与禁用态。

use crate::app::presentation::{MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{TextureButton, UiComposer, sliced_image};

#[derive(Clone, Copy)]
pub(crate) enum GameButtonImageMode {
    Stretch,
    Sliced { border: f32, corner_scale: f32 },
}

pub(crate) struct GameButtonSpec<'a> {
    pub label: &'a str,
    pub action: Option<UiAction>,
    pub normal: &'a Handle<Image>,
    pub hovered: &'a Handle<Image>,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub image_mode: GameButtonImageMode,
}

#[derive(Clone, Copy)]
pub(crate) enum GameButtonTone {
    Play,
    Pass,
    Hint,
}

impl<'a> GameButtonSpec<'a> {
    pub(crate) fn scaled(mut self, scale: f32) -> Self {
        self.width *= scale;
        self.height *= scale;
        self.font_size *= scale;
        self
    }

    pub(crate) fn compact(
        assets: &'a UiAssets,
        label: &'a str,
        action: Option<UiAction>,
        tone: GameButtonTone,
    ) -> Self {
        Self {
            width: 142.0,
            height: 48.0,
            ..Self::action(assets, label, action, tone)
        }
    }

    pub fn action(
        assets: &'a UiAssets,
        label: &'a str,
        action: Option<UiAction>,
        tone: GameButtonTone,
    ) -> Self {
        let (normal, hovered) = match tone {
            GameButtonTone::Play => (
                &assets.controls.game_play_button,
                &assets.controls.game_play_button_hover,
            ),
            GameButtonTone::Pass => (
                &assets.controls.game_pass_button,
                &assets.controls.game_pass_button_hover,
            ),
            GameButtonTone::Hint => (
                &assets.controls.game_hint_button,
                &assets.controls.game_hint_button_hover,
            ),
        };
        Self {
            label,
            action,
            normal,
            hovered,
            width: 164.0,
            height: 54.0,
            font_size: 18.0,
            image_mode: GameButtonImageMode::Stretch,
        }
    }
}

#[track_caller]
pub(crate) fn add_textured_game_button(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    spec: GameButtonSpec<'_>,
) -> (Entity, Entity) {
    let image = |texture: &Handle<Image>| match spec.image_mode {
        GameButtonImageMode::Stretch => ImageNode::new(texture.clone()),
        GameButtonImageMode::Sliced {
            border,
            corner_scale,
        } => sliced_image(texture.clone(), border, corner_scale),
    };
    let enabled = spec.action.is_some();
    TextureButton {
        node: Node {
            width: px(spec.width),
            height: px(spec.height),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        normal: image(spec.normal).with_color(Color::WHITE.with_alpha(if enabled {
            1.0
        } else {
            0.55
        })),
        hovered: image(spec.hovered),
        action: spec.action,
        label: Some(spec.label),
        font_size: spec.font_size,
        color: if enabled { TEXT } else { MUTED },
        icon: None,
    }
    .mount(commands, parent, &assets.font)
}
