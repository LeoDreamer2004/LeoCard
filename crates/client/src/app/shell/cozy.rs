//! 首页、顶栏和设置窗口共用的 CozyUI 纹理控件。

use super::{HomeHighlightKind, UiAction};
use crate::app::presentation::{MUTED, PanelSkin, TEXT, add_text, decorate_panel_skin};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

#[derive(Clone, Copy)]
pub(crate) enum CozyButtonVariant {
    Neutral,
    Primary,
    Cool,
    Danger,
}

pub(crate) fn add_cozy_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
    width: Val,
    height: f32,
) -> Entity {
    add_cozy_button_styled(
        commands,
        parent,
        Some(label),
        action,
        assets,
        width,
        height,
        None,
        CozyButtonVariant::Neutral,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the styled button keeps its action and geometry explicit"
)]
pub(crate) fn add_cozy_button_variant(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
    width: Val,
    height: f32,
    variant: CozyButtonVariant,
) -> Entity {
    add_cozy_button_styled(
        commands,
        parent,
        Some(label),
        action,
        assets,
        width,
        height,
        None,
        variant,
    )
}

pub(crate) fn add_cozy_disabled_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
    width: Val,
    height: f32,
) -> Entity {
    let mut image = ImageNode::new(assets.home.button.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    image.color = Color::WHITE.with_alpha(0.55);
    let button = commands
        .spawn((
            Node {
                width,
                height: px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            image,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(button);
    add_text(commands, button, label, 14.0, MUTED, assets);
    button
}

pub(crate) fn add_cozy_close_button(
    commands: &mut Commands,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            action,
            Node {
                width: px(40),
                height: px(40),
                ..default()
            },
            ImageNode::new(assets.home.close_button.clone()).with_mode(NodeImageMode::Stretch),
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
            ImageNode::new(assets.home.close_button_highlighted.clone())
                .with_mode(NodeImageMode::Stretch),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(overlay);
    commands.entity(button).insert(HomeHighlightKind::Button {
        overlay,
        arrows: None,
    });
    button
}

#[expect(
    clippy::too_many_arguments,
    reason = "the shared button keeps its action, size and optional icon explicit"
)]
pub(crate) fn add_cozy_button_with_icon(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
    width: Val,
    height: f32,
    icon: Option<Handle<Image>>,
) -> Entity {
    add_cozy_button_styled(
        commands,
        parent,
        Some(label),
        action,
        assets,
        width,
        height,
        icon,
        CozyButtonVariant::Neutral,
    )
}

pub(crate) fn add_cozy_icon_button(
    commands: &mut Commands,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
    icon: Handle<Image>,
    variant: CozyButtonVariant,
) -> Entity {
    add_cozy_button_styled(
        commands,
        parent,
        None,
        action,
        assets,
        px(38),
        38.0,
        Some(icon),
        variant,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the shared button keeps its action, size, optional icon and palette explicit"
)]
fn add_cozy_button_styled(
    commands: &mut Commands,
    parent: Entity,
    label: Option<&str>,
    action: UiAction,
    assets: &UiAssets,
    width: Val,
    height: f32,
    icon: Option<Handle<Image>>,
    variant: CozyButtonVariant,
) -> Entity {
    let (base_image, base_tint, hover_image, hover_tint, border) = match variant {
        CozyButtonVariant::Neutral => (
            assets.home.button.clone(),
            Color::WHITE,
            assets.home.purple_button_compact.clone(),
            Color::WHITE,
            32.0,
        ),
        CozyButtonVariant::Primary => (
            assets.home.purple_button_compact.clone(),
            Color::srgb(0.83, 0.80, 0.95),
            assets.home.purple_button_compact.clone(),
            Color::WHITE,
            32.0,
        ),
        CozyButtonVariant::Cool => (
            assets.home.cool_button.clone(),
            Color::WHITE,
            assets.home.cool_button_hover.clone(),
            Color::WHITE,
            32.0,
        ),
        CozyButtonVariant::Danger => (
            assets.home.danger_button.clone(),
            Color::WHITE,
            assets.home.danger_button_hover.clone(),
            Color::WHITE,
            32.0,
        ),
    };
    let mut base = ImageNode::new(base_image).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 0.55,
    }));
    base.visual_box = VisualBox::BorderBox;
    base.color = base_tint;
    let button = commands
        .spawn((
            Button,
            action,
            Node {
                width,
                height: px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(7),
                ..default()
            },
            base,
        ))
        .id();
    commands.entity(parent).add_child(button);

    let mut highlight =
        ImageNode::new(hover_image).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(border),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    highlight.visual_box = VisualBox::BorderBox;
    highlight.color = hover_tint;
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
            highlight,
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(overlay);
    if let Some(icon) = icon {
        let icon_size = if label.is_none() { 30.0 } else { 22.0 };
        let image = commands
            .spawn((
                Node {
                    width: px(icon_size),
                    height: px(icon_size),
                    ..default()
                },
                ImageNode::new(icon),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(image);
    }
    if let Some(label) = label {
        add_text(commands, button, label, 14.0, TEXT, assets);
    }
    commands.entity(button).insert(HomeHighlightKind::Button {
        overlay,
        arrows: None,
    });
    button
}

pub(crate) fn add_cozy_panel(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    assets: &UiAssets,
) -> Entity {
    add_cozy_panel_with_skin(commands, parent, node, PanelSkin::Window, assets)
}

pub(crate) fn add_cozy_panel_with_skin(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    let panel = commands.spawn(node).id();
    commands.entity(parent).add_child(panel);
    decorate_panel_skin(commands, panel, skin, assets);
    panel
}
