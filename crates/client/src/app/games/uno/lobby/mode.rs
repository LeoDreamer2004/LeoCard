use super::super::{UnoModeDropdownPanel, UnoUiAction};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{HomeHighlightKind, UiAction, add_cozy_panel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_uno::{Mode, UnoRuleSet};

pub(crate) fn render_uno_mode_dropdown(
    commands: &mut Commands,
    root: Entity,
    parent: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    open: bool,
    assets: &UiAssets,
) {
    let dropdown_accent = Color::srgb(0.85, 0.82, 1.0);
    let mode_label = |mode| match mode {
        Mode::Classic => "UNO",
        Mode::NoMercy => "No Mercy",
        Mode::Flip => "UNO FLIP",
    };

    if open && can_configure {
        let dismiss = commands
            .spawn((
                Button,
                UiAction::Uno(UnoUiAction::CloseModeMenu),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                GlobalZIndex(1850),
                FocusPolicy::Block,
            ))
            .id();
        commands.entity(root).add_child(dismiss);
    }

    let selector = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(190),
            height: px(40),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    commands.entity(selector).insert(GlobalZIndex(1870));

    let mut trigger_image =
        ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    trigger_image.visual_box = VisualBox::BorderBox;
    if !can_configure {
        trigger_image.color = Color::WHITE.with_alpha(0.72);
    }
    let mut trigger = commands.spawn((
        Node {
            width: percent(100),
            height: px(40),
            padding: UiRect::horizontal(px(13)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        trigger_image,
    ));
    if can_configure {
        trigger.insert((Button, UiAction::Uno(UnoUiAction::ToggleModeMenu)));
    } else {
        trigger.insert(FocusPolicy::Block);
    }
    let trigger = trigger.id();
    commands.entity(selector).add_child(trigger);
    if can_configure {
        let mut hover_image = ImageNode::new(assets.home.focused_input.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55,
            }),
        );
        hover_image.visual_box = VisualBox::BorderBox;
        let hover = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                hover_image,
                Visibility::Hidden,
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(trigger).add_child(hover);
        commands.entity(trigger).insert(HomeHighlightKind::Button {
            overlay: hover,
            arrows: None,
        });
    }
    let label = add_text(
        commands,
        trigger,
        mode_label(rules.mode),
        14.0,
        TEXT,
        assets,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
    let arrow = add_text(
        commands,
        trigger,
        if can_configure {
            if open { "▲" } else { "▼" }
        } else {
            "—"
        },
        12.0,
        if can_configure {
            dropdown_accent
        } else {
            MUTED
        },
        assets,
    );
    commands.entity(arrow).insert(FocusPolicy::Pass);

    if !open || !can_configure {
        return;
    }
    let menu = add_cozy_panel(
        commands,
        selector,
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(44),
            width: px(190),
            padding: UiRect::all(px(7)),
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        },
        assets,
    );
    commands
        .entity(menu)
        .insert((UnoModeDropdownPanel, GlobalZIndex(1890), FocusPolicy::Block));
    for (mode, label) in [
        (Mode::Classic, "UNO"),
        (Mode::NoMercy, "No Mercy"),
        (Mode::Flip, "UNO FLIP"),
    ] {
        let selected = mode == rules.mode;
        let mut option_image = ImageNode::new(if selected {
            assets.home.purple_button_compact.clone()
        } else {
            assets.home.button.clone()
        })
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
        option_image.visual_box = VisualBox::BorderBox;
        let option = commands
            .spawn((
                Button,
                UiAction::Uno(UnoUiAction::UpdateRules(UnoRuleSet { mode, ..rules })),
                Node {
                    width: percent(100),
                    height: px(38),
                    padding: UiRect::horizontal(px(11)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                option_image,
            ))
            .id();
        commands.entity(menu).add_child(option);
        if !selected {
            let mut hover_image = ImageNode::new(assets.home.purple_button_compact.clone())
                .with_mode(NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(32.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 0.55,
                }));
            hover_image.visual_box = VisualBox::BorderBox;
            let hover = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    hover_image,
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(option).add_child(hover);
            commands.entity(option).insert(HomeHighlightKind::Button {
                overlay: hover,
                arrows: None,
            });
        }
        let label = add_text(commands, option, label, 13.5, TEXT, assets);
        commands.entity(label).insert(FocusPolicy::Pass);
    }
}
