//! 牌桌操作按钮共用的纹理、悬停层与禁用态。

use crate::app::presentation::ButtonHighlight;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

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
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(overlay);
        commands.entity(button).insert(ButtonHighlight::Button {
            overlay,
            arrows: None,
        });
    } else {
        commands.entity(button).insert(FocusPolicy::Pass);
    }
    let label = add_text(
        commands,
        button,
        spec.label,
        spec.font_size,
        if enabled { TEXT } else { MUTED },
        assets,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
    (button, label)
}
