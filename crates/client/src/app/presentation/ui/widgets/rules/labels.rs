use super::*;

pub fn add_rule_help(commands: &mut Commands, parent: Entity, help: &str, assets: &UiAssets) {
    let question = commands
        .spawn((
            Button,
            Node {
                width: px(22),
                height: px(22),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(MUTED.with_alpha(0.8)),
        ))
        .id();
    commands.entity(parent).add_child(question);
    add_text(commands, question, "?", 14.0, MUTED, assets);

    let tooltip = spawn_node(
        commands,
        question,
        Node {
            position_type: PositionType::Absolute,
            right: px(28),
            top: px(-7),
            width: px(270),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        BorderColor::all(ACCENT.with_alpha(0.7)),
        GlobalZIndex(1000),
    ));
    add_text(commands, tooltip, help, 12.0, TEXT, assets);
    commands.entity(question).insert(RuleHelp { tooltip });
}

pub fn suit_comparison_label(comparison: SuitComparison) -> &'static str {
    match comparison {
        SuitComparison::HighestCard => "极大法",
        SuitComparison::Lexicographic => "逐项法",
        SuitComparison::SumPoints => "记点法",
    }
}

pub fn time_control_label(control: TimeControl) -> &'static str {
    match control {
        TimeControl::Unlimited => "不限时",
        TimeControl::FivePlusTen => "5 + 10",
        TimeControl::FivePlusThirty => "5 + 30",
        TimeControl::FifteenPlusThirty => "15 + 30",
        TimeControl::ThirtyPlusSixty => "30 + 60",
    }
}

pub fn previous_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => None,
        TimeControl::FivePlusThirty => Some(TimeControl::FivePlusTen),
        TimeControl::FifteenPlusThirty => Some(TimeControl::FivePlusThirty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::Unlimited => Some(TimeControl::ThirtyPlusSixty),
    }
}

pub fn next_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => Some(TimeControl::FivePlusThirty),
        TimeControl::FivePlusThirty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::FifteenPlusThirty => Some(TimeControl::ThirtyPlusSixty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::Unlimited),
        TimeControl::Unlimited => None,
    }
}

pub fn previous_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::SumPoints,
        SuitComparison::Lexicographic => SuitComparison::HighestCard,
        SuitComparison::SumPoints => SuitComparison::Lexicographic,
    }
}

pub fn next_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::Lexicographic,
        SuitComparison::Lexicographic => SuitComparison::SumPoints,
        SuitComparison::SumPoints => SuitComparison::HighestCard,
    }
}
