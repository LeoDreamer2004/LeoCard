//! 首页、顶栏和设置窗口共用的 CozyUI 纹理控件。

use super::super::UiAction;
use crate::app::presentation::{
    MUTED, PanelSkin, TEXT, TextureButton, UiComposer, decorate_panel_skin, sliced_image,
    spawn_node, text_scene,
};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) enum CozyButtonVariant {
    Neutral,
    Primary,
    Cool,
    Danger,
}

#[track_caller]
pub(crate) fn add_cozy_button(
    commands: &mut UiComposer,
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
#[track_caller]
pub(crate) fn add_cozy_button_variant(
    commands: &mut UiComposer,
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

#[track_caller]
pub(crate) fn add_cozy_disabled_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
    width: Val,
    height: f32,
) -> Entity {
    TextureButton {
        node: button_node(width, height),
        normal: sliced_image(assets.home.button.clone(), 32.0, 0.55)
            .with_color(Color::WHITE.with_alpha(0.55)),
        hovered: ImageNode::default(),
        action: None,
        label: Some(label),
        font_size: 14.0,
        color: MUTED,
        icon: None,
    }
    .mount(commands, parent, assets)
    .0
}

#[track_caller]
pub(crate) fn add_cozy_close_button(
    commands: &mut UiComposer,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
) -> Entity {
    add_cozy_control_button(
        commands,
        parent,
        action,
        assets,
        [
            assets.home.close_button.clone(),
            assets.home.close_button_highlighted.clone(),
        ],
    )
}

#[track_caller]
pub(crate) fn add_cozy_refresh_button(
    commands: &mut UiComposer,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
) -> Entity {
    add_cozy_control_button(
        commands,
        parent,
        action,
        assets,
        [
            assets.home.refresh_button.clone(),
            assets.home.refresh_button_highlighted.clone(),
        ],
    )
}

#[track_caller]
fn add_cozy_control_button(
    commands: &mut UiComposer,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
    [normal, hovered]: [Handle<Image>; 2],
) -> Entity {
    TextureButton {
        node: button_node(px(40), 40.0),
        normal: ImageNode::new(normal),
        hovered: ImageNode::new(hovered),
        action: Some(action),
        label: None,
        font_size: 14.0,
        color: TEXT,
        icon: None,
    }
    .mount(commands, parent, assets)
    .0
}

#[expect(
    clippy::too_many_arguments,
    reason = "the shared button keeps its action, size and optional icon explicit"
)]
#[track_caller]
pub(crate) fn add_cozy_button_with_icon(
    commands: &mut UiComposer,
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

#[track_caller]
pub(crate) fn add_cozy_icon_button(
    commands: &mut UiComposer,
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
#[track_caller]
fn add_cozy_button_styled(
    commands: &mut UiComposer,
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
    TextureButton {
        node: button_node(width, height),
        normal: sliced_image(base_image, border, 0.55).with_color(base_tint),
        hovered: sliced_image(hover_image, border, 0.55).with_color(hover_tint),
        action: Some(action),
        label,
        font_size: 14.0,
        color: TEXT,
        icon: icon.map(|icon| (icon, if label.is_none() { 30.0 } else { 22.0 })),
    }
    .mount(commands, parent, assets)
    .0
}

fn button_node(width: Val, height: f32) -> Node {
    Node {
        width,
        height: px(height),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: px(7),
        ..default()
    }
}

#[track_caller]
pub(crate) fn add_cozy_panel(
    commands: &mut UiComposer,
    parent: Entity,
    node: Node,
    assets: &UiAssets,
) -> Entity {
    add_cozy_panel_with_skin(commands, parent, node, PanelSkin::Window, assets)
}

#[track_caller]
pub(crate) fn add_cozy_panel_with_skin(
    commands: &mut UiComposer,
    parent: Entity,
    node: Node,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    let panel = spawn_node(commands, parent, node, None);
    decorate_panel_skin(commands, panel, skin, assets);
    panel
}

pub(crate) fn add_cozy_labeled_icon_button(
    commands: &mut UiComposer,
    parent: Entity,
    action: UiAction,
    assets: &UiAssets,
    icon: Handle<Image>,
    variant: CozyButtonVariant,
    label: &str,
) {
    let column = commands.mount(
        parent,
        format!("entry:{label}"),
        bsn! {
            Node {
                width: px(44), flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center, row_gap: px(1),
            }
        },
    );
    add_cozy_icon_button(commands, column, action, assets, icon, variant);
    commands.node(column, text_scene(assets, label, 12.0, TEXT));
}
