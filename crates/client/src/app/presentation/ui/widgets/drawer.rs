use super::add_text;
use super::spawn_node;
use crate::app::presentation::{ACCENT, ButtonHighlight, MUTED, UiComposer};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;

pub(crate) struct DrawerSwitch<'a> {
    pub short: &'a str,
    pub label: &'a str,
    pub enabled: bool,
    pub action: UiAction,
}

pub(crate) struct SwitchDrawer<'a> {
    pub expanded: bool,
    pub toggle_action: UiAction,
    pub switches: &'a [DrawerSwitch<'a>],
}

/// Anchor to a full-width game viewport, not a centered fixed-width table.
pub(crate) fn add_switch_drawer(
    commands: &mut UiComposer,
    viewport: Entity,
    spec: SwitchDrawer<'_>,
    assets: &UiAssets,
) -> Entity {
    let expanded = spec.expanded;
    let mut drawer_image =
        ImageNode::new(assets.home.panel.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(80.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.20,
        }));
    drawer_image.visual_box = VisualBox::BorderBox;
    let drawer = spawn_node(
        commands,
        viewport,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(82),
            width: px(if expanded { 142 } else { 38 }),
            padding: UiRect::axes(px(if expanded { 13 } else { 3 }), px(10)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    commands
        .entity(drawer)
        .insert((drawer_image, GlobalZIndex(1200)));
    for option in spec.switches {
        let DrawerSwitch {
            short,
            label,
            enabled,
            action,
        } = option;
        let row = commands.child(
            drawer,
            (
                Button,
                action.clone(),
                Node {
                    width: percent(100),
                    height: px(31),
                    align_items: AlignItems::Center,
                    justify_content: if expanded {
                        JustifyContent::SpaceBetween
                    } else {
                        JustifyContent::Center
                    },
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ),
        );
        add_text(
            commands,
            row,
            if expanded { *label } else { *short },
            if expanded { 14.0 } else { 17.0 },
            if *enabled { ACCENT } else { MUTED },
            assets,
        );
        if expanded {
            commands.child(
                row,
                (
                    Node {
                        width: px(20),
                        height: px(20),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode::new(if *enabled {
                        assets.home.checkbox_selected.clone()
                    } else {
                        assets.home.checkbox.clone()
                    }),
                    Pickable::IGNORE,
                ),
            );
        }
    }
    let arrow_row = spawn_node(
        commands,
        drawer,
        Node {
            width: percent(100),
            height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let arrow = commands.child(
        arrow_row,
        (
            Button,
            spec.toggle_action,
            Node {
                width: px(20),
                height: px(28),
                ..default()
            },
            ImageNode::new(if expanded {
                assets.home.rule_left.clone()
            } else {
                assets.home.rule_right.clone()
            }),
        ),
    );
    let hover = commands.child(
        arrow,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            ImageNode::new(if expanded {
                assets.home.rule_left_highlighted.clone()
            } else {
                assets.home.rule_right_highlighted.clone()
            }),
            Visibility::Hidden,
            Pickable::IGNORE,
        ),
    );
    commands.entity(arrow).insert(ButtonHighlight {
        overlay: hover,
        arrows: None,
    });
    drawer
}
