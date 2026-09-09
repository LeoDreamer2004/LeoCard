use super::super::{UnoModeDropdownPanel, UnoUiAction};
use crate::app::presentation::{
    BackgroundButtonTint, ButtonTint, MUTED, TEXT, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
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
    let dropdown_accent = Color::srgb(0.24, 0.90, 0.86);
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

    let mut trigger = commands.spawn((
        Node {
            width: percent(100),
            height: px(40),
            padding: UiRect::horizontal(px(13)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(if can_configure {
            Color::srgba(0.04, 0.48, 0.50, 0.48)
        } else {
            Color::srgba(0.30, 0.32, 0.33, 0.42)
        }),
        BorderColor::all(if can_configure {
            Color::srgba(0.30, 0.92, 0.88, 0.62)
        } else {
            Color::srgba(0.72, 0.74, 0.74, 0.28)
        }),
        BoxShadow::new(Color::BLACK.with_alpha(0.26), px(1), px(3), px(0), px(6)),
    ));
    if can_configure {
        trigger.insert((
            Button,
            BackgroundButtonTint,
            UiAction::Uno(UnoUiAction::ToggleModeMenu),
            ButtonTint {
                normal: Color::srgba(0.04, 0.48, 0.50, 0.48),
                hovered: Color::srgba(0.06, 0.68, 0.70, 0.64),
                pressed: Color::srgba(0.03, 0.36, 0.40, 0.42),
            },
        ));
    } else {
        trigger.insert(FocusPolicy::Block);
    }
    let trigger = trigger.id();
    commands.entity(selector).add_child(trigger);
    let label = add_text(
        commands,
        trigger,
        mode_label(rules.mode),
        14.0,
        Color::WHITE,
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
    let menu = spawn_node(
        commands,
        selector,
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(44),
            width: px(190),
            padding: UiRect::all(px(5)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        Some(Color::srgba(0.025, 0.090, 0.095, 0.88)),
    );
    commands.entity(menu).insert((
        UnoModeDropdownPanel,
        BorderColor::all(Color::srgba(0.30, 0.92, 0.88, 0.38)),
        BoxShadow::new(Color::BLACK.with_alpha(0.46), px(2), px(6), px(0), px(10)),
        GlobalZIndex(1890),
        FocusPolicy::Block,
    ));
    for (mode, label) in [
        (Mode::Classic, "UNO"),
        (Mode::NoMercy, "No Mercy"),
        (Mode::Flip, "UNO FLIP"),
    ] {
        let selected = mode == rules.mode;
        let option = commands
            .spawn((
                Button,
                BackgroundButtonTint,
                UiAction::Uno(UnoUiAction::UpdateRules(UnoRuleSet { mode, ..rules })),
                ButtonTint {
                    normal: if selected {
                        Color::srgba(0.05, 0.62, 0.62, 0.38)
                    } else {
                        Color::srgba(0.22, 0.66, 0.66, 0.10)
                    },
                    hovered: Color::srgba(0.08, 0.76, 0.74, 0.48),
                    pressed: Color::srgba(0.03, 0.42, 0.44, 0.34),
                },
                Node {
                    width: percent(100),
                    height: px(36),
                    padding: UiRect::horizontal(px(11)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BackgroundColor(if selected {
                    Color::srgba(0.05, 0.62, 0.62, 0.38)
                } else {
                    Color::srgba(0.22, 0.66, 0.66, 0.10)
                }),
            ))
            .id();
        commands.entity(menu).add_child(option);
        let label = add_text(
            commands,
            option,
            if selected {
                format!("✓  {label}")
            } else {
                format!("   {label}")
            },
            13.5,
            if selected { dropdown_accent } else { TEXT },
            assets,
        );
        commands.entity(label).insert(FocusPolicy::Pass);
    }
}
