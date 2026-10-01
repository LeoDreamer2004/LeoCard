use super::super::super::{ConnectionUiAction, UiAction};
use crate::app::presentation::{ButtonArrows, ButtonHighlight, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

pub(super) fn home_panel_image(assets: &UiAssets) -> ImageNode {
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

pub(super) fn home_game_card_image(texture: Handle<Image>) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn home_panel(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    assets: &UiAssets,
) -> Entity {
    let entity = commands.spawn(node).id();
    commands.entity(parent).add_child(entity);
    let background = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            home_panel_image(assets),
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(entity).add_child(background);
    entity
}

pub(super) fn home_button_image(assets: &UiAssets) -> ImageNode {
    let mut image = ImageNode::new(assets.home.button.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn add_home_purple_overlay(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
    visible: bool,
    compact: bool,
) -> Entity {
    let texture = if compact {
        &assets.home.purple_button_compact
    } else {
        &assets.home.purple_button
    };
    let mut image =
        ImageNode::new(texture.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: Vec2::new(32.0, 32.0),
                max_inset: Vec2::new(32.0, 32.0),
            },
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    image.visual_box = VisualBox::BorderBox;
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
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            image,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    overlay
}

pub(super) fn add_home_animated_arrow(
    commands: &mut Commands,
    parent: Entity,
    owner: Entity,
    assets: &UiAssets,
    left: bool,
    size: Vec2,
) -> Entity {
    let mut image = ImageNode::new(assets.home.button_arrows.clone());
    let x = if left { 8.0 } else { 136.0 };
    image.rect = Some(Rect::from_corners(
        Vec2::new(x, 8.0),
        Vec2::new(x + 112.0, 64.0),
    ));
    let entity = commands
        .spawn((
            Node {
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            image,
            ButtonArrows {
                left,
                owner,
                elapsed: 0.0,
            },
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub(super) fn home_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: ConnectionUiAction,
    assets: &UiAssets,
    primary: bool,
    width: Val,
) {
    let entity = commands
        .spawn((
            Button,
            UiAction::Connection(action),
            Node {
                width,
                height: px(43),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(5),
                ..default()
            },
            home_button_image(assets),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    let overlay = add_home_purple_overlay(commands, entity, assets, false, true);
    let left_arrow = primary.then(|| {
        add_home_animated_arrow(
            commands,
            entity,
            entity,
            assets,
            true,
            Vec2::new(64.0, 40.0),
        )
    });
    add_text(commands, entity, label, 14.0, TEXT, assets);
    let right_arrow = primary.then(|| {
        add_home_animated_arrow(
            commands,
            entity,
            entity,
            assets,
            false,
            Vec2::new(64.0, 40.0),
        )
    });
    commands.entity(entity).insert(ButtonHighlight::Button {
        overlay,
        arrows: left_arrow
            .zip(right_arrow)
            .map(|(left, right)| [left, right]),
    });
}
