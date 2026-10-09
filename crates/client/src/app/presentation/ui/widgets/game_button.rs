//! 牌桌操作按钮共用的纹理、悬停层与禁用态。

use crate::app::presentation::ButtonHighlight;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;

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

fn game_button_image(texture: &Handle<Image>, mode: GameButtonImageMode, alpha: f32) -> ImageNode {
    let mode = match mode {
        GameButtonImageMode::Stretch => NodeImageMode::Stretch,
        GameButtonImageMode::Sliced {
            border,
            corner_scale,
        } => NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(border),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: corner_scale,
        }),
    };
    let mut image = ImageNode::new(texture.clone()).with_mode(mode);
    image.visual_box = VisualBox::BorderBox;
    image.color = Color::WHITE.with_alpha(alpha);
    image
}

pub(crate) fn add_textured_game_button(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
    spec: GameButtonSpec<'_>,
) -> (Entity, Entity) {
    let enabled = spec.action.is_some();
    let button = commands
        .spawn((
            Node {
                width: px(spec.width),
                height: px(spec.height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            game_button_image(
                spec.normal,
                spec.image_mode,
                if enabled { 1.0 } else { 0.55 },
            ),
        ))
        .id();
    commands.entity(parent).add_child(button);
    if let Some(action) = spec.action {
        commands.entity(button).insert((Button, action));
        let overlay = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                game_button_image(spec.hovered, spec.image_mode, 1.0),
                Visibility::Hidden,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(button).add_child(overlay);
        commands.entity(button).insert(ButtonHighlight {
            overlay,
            arrows: None,
        });
    } else {
        commands.entity(button).insert(Pickable::IGNORE);
    }
    let label = add_text(
        commands,
        button,
        spec.label,
        spec.font_size,
        if enabled { TEXT } else { MUTED },
        assets,
    );
    commands.entity(label).insert(Pickable::IGNORE);
    (button, label)
}
