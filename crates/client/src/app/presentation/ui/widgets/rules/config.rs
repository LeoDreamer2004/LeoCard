use super::super::{add_text, spawn_node};
use super::add_rule_help;
use crate::app::presentation::ButtonHighlight;
use crate::app::presentation::{MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};

pub(crate) trait EditableRuleSet: Copy {
    const ROW_HEIGHT: f32;
    const VALUE_WIDTH: f32;

    fn update_action(self) -> UiAction;
}

pub(crate) struct RuleConfigRow<'a, R> {
    pub label: &'a str,
    pub value: String,
    pub help: &'a str,
    pub editable: bool,
    pub previous: Option<R>,
    pub next: Option<R>,
}

pub(crate) fn add_rule_config_row<R: EditableRuleSet>(
    commands: &mut Commands,
    parent: Entity,
    spec: RuleConfigRow<'_, R>,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(R::ROW_HEIGHT.max(44.0)),
            position_type: PositionType::Relative,
            padding: UiRect::vertical(px(5)),
            border: UiRect::bottom(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands
        .entity(row)
        .insert(BorderColor::all(Color::srgba(0.70, 0.69, 0.77, 0.25)));
    add_text(commands, row, spec.label, 15.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    if spec.editable {
        add_rule_step_button(commands, controls, true, spec.previous, assets);
    }
    let mut input_image =
        ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    input_image.visual_box = VisualBox::BorderBox;
    let value_box = commands
        .spawn((
            Node {
                min_width: px(R::VALUE_WIDTH),
                min_height: px(34),
                padding: UiRect::horizontal(px(7)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            input_image,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(controls).add_child(value_box);
    add_text(commands, value_box, spec.value, 15.0, TEXT, assets);
    if spec.editable {
        add_rule_step_button(commands, controls, false, spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

fn add_rule_step_button<R: EditableRuleSet>(
    commands: &mut Commands,
    parent: Entity,
    left: bool,
    rules: Option<R>,
    assets: &UiAssets,
) {
    let (normal, highlighted) = if left {
        (&assets.home.rule_left, &assets.home.rule_left_highlighted)
    } else {
        (&assets.home.rule_right, &assets.home.rule_right_highlighted)
    };
    let mut image = ImageNode::new(normal.clone()).with_mode(NodeImageMode::Stretch);
    if rules.is_none() {
        image.color = Color::WHITE.with_alpha(0.35);
    }
    let node = Node {
        width: px(24),
        height: px(34),
        flex_shrink: 0.0,
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((Button, rules.update_action(), node, image))
            .id()
    } else {
        commands.spawn((node, image, FocusPolicy::Pass)).id()
    };
    commands.entity(parent).add_child(entity);
    if rules.is_some() {
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
                ImageNode::new(highlighted.clone()).with_mode(NodeImageMode::Stretch),
                Visibility::Hidden,
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(entity).add_child(hover);
        commands.entity(entity).insert(ButtonHighlight {
            overlay: hover,
            arrows: None,
        });
    }
}
