//! 七鬼五二三牌桌的小面板与操作按钮纹理。

use super::QiGui523Assets;
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{HomeHighlightKind, UiAction};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

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
    game_assets: &QiGui523Assets,
) -> (Entity, Entity) {
    let (normal, hovered) = match tone {
        QiGuiButtonTone::Play => (
            game_assets.action_play.clone(),
            game_assets.action_play_hover.clone(),
        ),
        QiGuiButtonTone::Pass => (
            game_assets.action_pass.clone(),
            game_assets.action_pass_hover.clone(),
        ),
        QiGuiButtonTone::Hint => (
            game_assets.action_hint.clone(),
            game_assets.action_hint_hover.clone(),
        ),
    };
    let flat = |texture| {
        let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Stretch);
        image.visual_box = VisualBox::BorderBox;
        image
    };
    let button = commands
        .spawn((
            Button,
            action,
            Node {
                width: px(164),
                height: px(54),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            flat(normal),
        ))
        .id();
    commands.entity(parent).add_child(button);
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
            flat(hovered),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(overlay);
    let label = add_text(commands, button, label, 18.0, TEXT, assets);
    commands.entity(label).insert(FocusPolicy::Pass);
    commands.entity(button).insert(HomeHighlightKind::Button {
        overlay,
        arrows: None,
    });
    (button, label)
}
