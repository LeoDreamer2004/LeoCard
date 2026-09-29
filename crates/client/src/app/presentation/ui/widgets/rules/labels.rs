use super::super::add_text;
use crate::app::presentation::{RuleHelp, TEXT};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_qigui523::{SuitComparison, TimeControl};

pub(super) fn add_rule_help(
    commands: &mut Commands,
    parent: Entity,
    help: &str,
    assets: &UiAssets,
) {
    let question = commands
        .spawn((
            Button,
            Node {
                width: px(26),
                height: px(26),
                position_type: PositionType::Relative,
                ..default()
            },
            ImageNode::new(assets.home.help_question.clone()),
        ))
        .id();
    commands.entity(parent).add_child(question);

    let mut tooltip_image =
        ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    tooltip_image.visual_box = VisualBox::BorderBox;
    let tooltip = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(30),
                top: px(-7),
                width: px(290),
                padding: UiRect::all(px(15)),
                ..default()
            },
            tooltip_image,
        ))
        .id();
    commands.entity(question).add_child(tooltip);
    commands
        .entity(tooltip)
        .insert((Visibility::Hidden, GlobalZIndex(1000)));
    add_text(commands, tooltip, help, 12.0, TEXT, assets);
    commands.entity(question).insert(RuleHelp { tooltip });
}

pub(crate) fn suit_comparison_label(comparison: SuitComparison) -> &'static str {
    match comparison {
        SuitComparison::HighestCard => "极大法",
        SuitComparison::Lexicographic => "逐项法",
        SuitComparison::SumPoints => "记点法",
    }
}

pub(crate) fn time_control_label(control: TimeControl) -> &'static str {
    match control {
        TimeControl::Unlimited => "不限时",
        TimeControl::FivePlusTen => "5 + 10",
        TimeControl::FivePlusThirty => "5 + 30",
        TimeControl::FifteenPlusThirty => "15 + 30",
        TimeControl::ThirtyPlusSixty => "30 + 60",
    }
}

pub(crate) fn previous_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => None,
        TimeControl::FivePlusThirty => Some(TimeControl::FivePlusTen),
        TimeControl::FifteenPlusThirty => Some(TimeControl::FivePlusThirty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::Unlimited => Some(TimeControl::ThirtyPlusSixty),
    }
}

pub(crate) fn next_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => Some(TimeControl::FivePlusThirty),
        TimeControl::FivePlusThirty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::FifteenPlusThirty => Some(TimeControl::ThirtyPlusSixty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::Unlimited),
        TimeControl::Unlimited => None,
    }
}

pub(crate) fn previous_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::SumPoints,
        SuitComparison::Lexicographic => SuitComparison::HighestCard,
        SuitComparison::SumPoints => SuitComparison::Lexicographic,
    }
}

pub(crate) fn next_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::Lexicographic,
        SuitComparison::Lexicographic => SuitComparison::SumPoints,
        SuitComparison::SumPoints => SuitComparison::HighestCard,
    }
}
