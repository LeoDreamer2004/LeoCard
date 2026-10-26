use super::{HelpTooltipStyle, add_help_tooltip};
use crate::{TextureButton, UiComposer, add_label, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct RuleRowStyle<'a> {
    pub font: &'a Handle<Font>,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub background: ImageNode,
    pub arrows: [(ImageNode, ImageNode); 2],
    pub help: HelpTooltipStyle<'a>,
    pub row_height: f32,
    pub value_width: f32,
}

pub struct RuleRow<'a, A> {
    pub label: &'a str,
    pub value: String,
    pub help: &'a str,
    pub editable: bool,
    pub previous: Option<A>,
    pub next: Option<A>,
}

pub fn add_rule_row<A: Component + Clone>(
    commands: &mut UiComposer,
    parent: Entity,
    spec: RuleRow<'_, A>,
    style: &RuleRowStyle,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(style.row_height.max(44.0)),
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
    commands.entity(row).insert(BorderColor::all(style.border));
    add_label(commands, row, spec.label, 15.0, style.muted, style.font);
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
        add_rule_step_button(commands, controls, true, spec.previous, style);
    }
    let value_box = commands.child(
        controls,
        (
            Node {
                min_width: px(style.value_width),
                min_height: px(34),
                padding: UiRect::horizontal(px(7)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            style.background.clone(),
            Pickable::IGNORE,
        ),
    );
    add_label(
        commands, value_box, spec.value, 15.0, style.text, style.font,
    );
    if spec.editable {
        add_rule_step_button(commands, controls, false, spec.next, style);
    }
    add_help_tooltip(commands, controls, spec.help, style.help.clone());
}

fn add_rule_step_button<A: Component + Clone>(
    commands: &mut UiComposer,
    parent: Entity,
    left: bool,
    rules: Option<A>,
    style: &RuleRowStyle,
) {
    let (normal, highlighted) = &style.arrows[usize::from(!left)];
    let mut image = normal.clone();
    if rules.is_none() {
        image.color = Color::WHITE.with_alpha(0.35);
    }
    let node = Node {
        width: px(24),
        height: px(34),
        flex_shrink: 0.0,
        ..default()
    };
    TextureButton {
        node,
        normal: image,
        hovered: highlighted.clone(),
        action: rules,
        label: None,
        font_size: 14.0,
        color: style.text,
        icon: None,
    }
    .mount(commands, parent, style.font);
}
