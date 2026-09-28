//! 七鬼五二三牌桌的小面板与操作按钮纹理。

use crate::app::presentation::{GameButtonImageMode, GameButtonSpec, add_textured_game_button};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::VisualBox;

#[derive(Clone, Copy)]
pub(super) enum QiGuiButtonTone {
    Play,
    Pass,
    Hint,
}

pub(super) fn qigui_plate_image(assets: &UiAssets) -> ImageNode {
    let mut image = ImageNode::new(assets.home.game_card.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(22.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.42,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn qigui_panel_image(assets: &UiAssets) -> ImageNode {
    let mut image =
        ImageNode::new(assets.home.panel.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(80.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.42,
        }));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn add_qigui_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    tone: QiGuiButtonTone,
    assets: &UiAssets,
) -> (Entity, Entity) {
    let (normal, hovered) = match tone {
        QiGuiButtonTone::Play => (
            &assets.controls.game_play_button,
            &assets.controls.game_play_button_hover,
        ),
        QiGuiButtonTone::Pass => (
            &assets.controls.game_pass_button,
            &assets.controls.game_pass_button_hover,
        ),
        QiGuiButtonTone::Hint => (
            &assets.controls.game_hint_button,
            &assets.controls.game_hint_button_hover,
        ),
    };
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action: Some(action),
            normal,
            hovered,
            width: 164.0,
            height: 54.0,
            font_size: 18.0,
            image_mode: GameButtonImageMode::Stretch,
        },
    )
}
