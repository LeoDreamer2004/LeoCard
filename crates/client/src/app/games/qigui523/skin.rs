//! 七鬼五二三牌桌的小面板与操作按钮纹理。

use crate::app::presentation::{GameButtonSpec, add_textured_game_button};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::VisualBox;

pub(super) use crate::app::presentation::GameButtonTone as QiGuiButtonTone;

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

pub(super) fn add_qigui_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    tone: QiGuiButtonTone,
    assets: &UiAssets,
) -> (Entity, Entity) {
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec::action(assets, label, Some(action), tone),
    )
}
