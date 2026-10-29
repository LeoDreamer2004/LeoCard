use crate::{ButtonHighlight, UiComposer, add_label, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub struct SwitchDrawerStyle<'a> {
    pub scale: f32,
    pub font: &'a Handle<Font>,
    pub background: ImageNode,
    pub checkbox: Handle<Image>,
    pub checkbox_selected: Handle<Image>,
    pub left: Handle<Image>,
    pub left_hovered: Handle<Image>,
    pub right: Handle<Image>,
    pub right_hovered: Handle<Image>,
    pub active: Color,
    pub muted: Color,
}

pub struct DrawerSwitch<'a, A> {
    pub short: &'a str,
    pub label: &'a str,
    pub enabled: bool,
    pub action: A,
}

pub struct SwitchDrawer<'a, A> {
    pub expanded: bool,
    pub toggle_action: A,
    pub switches: &'a [DrawerSwitch<'a, A>],
}

/// Anchor to a full-width game viewport, not a centered fixed-width table.
pub fn spawn_switch_drawer<A: Component + Clone>(
    commands: &mut UiComposer,
    viewport: Entity,
    spec: SwitchDrawer<'_, A>,
    style: &SwitchDrawerStyle,
) -> Entity {
    let expanded = spec.expanded;
    // A zero-sized anchor scales around the viewport edge, keeping the drawer flush with it.
    let anchor = spawn_node(
        commands,
        viewport,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(82),
            width: px(0),
            height: px(0),
            ..default()
        },
        None,
    );
    commands
        .entity(anchor)
        .insert(UiTransform::from_scale(Vec2::splat(style.scale)));
    let drawer = spawn_node(
        commands,
        anchor,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(0),
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
        .insert((style.background.clone(), GlobalZIndex(1200)));
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
        add_label(
            commands,
            row,
            if expanded { *label } else { *short },
            if expanded { 14.0 } else { 17.0 },
            if *enabled { style.active } else { style.muted },
            style.font,
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
                        style.checkbox_selected.clone()
                    } else {
                        style.checkbox.clone()
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
                style.left.clone()
            } else {
                style.right.clone()
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
                style.left_hovered.clone()
            } else {
                style.right_hovered.clone()
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
