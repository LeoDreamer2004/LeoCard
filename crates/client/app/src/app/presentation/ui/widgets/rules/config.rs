use super::rule_help_style;
use crate::app::presentation::{MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{RuleRow, RuleRowStyle, UiComposer, add_rule_row, sliced_image};

pub(crate) trait EditableRuleSet: Copy {
    const ROW_HEIGHT: f32;
    const VALUE_WIDTH: f32;

    fn update_action(self) -> UiAction;
}

pub(crate) struct RuleConfigRow<'a, R> {
    pub label: &'a str,
    pub value: String,
    pub help: &'a str,
    /// Read-only rows and value limits use `None`; both arrows remain visible.
    pub previous: Option<R>,
    pub next: Option<R>,
}

impl<R> RuleConfigRow<'_, R> {
    /// Inapplicable rules show one shared label and cannot change in either direction.
    pub(crate) fn applicable(mut self, applicable: bool) -> Self {
        if !applicable {
            self.value = "不适用".to_owned();
            self.previous = None;
            self.next = None;
        }
        self
    }
}

pub(crate) fn add_rule_config_row<R: EditableRuleSet>(
    commands: &mut UiComposer,
    parent: Entity,
    spec: RuleConfigRow<'_, R>,
    assets: &UiAssets,
) {
    add_rule_row(
        commands,
        parent,
        RuleRow {
            label: spec.label,
            value: spec.value,
            help: spec.help,
            previous: spec.previous.map(EditableRuleSet::update_action),
            next: spec.next.map(EditableRuleSet::update_action),
        },
        &RuleRowStyle {
            font: &assets.font,
            text: TEXT,
            muted: MUTED,
            border: Color::srgba(0.70, 0.69, 0.77, 0.25),
            background: sliced_image(assets.home.input.clone(), 32.0, 0.55),
            arrows: [
                (
                    ImageNode::new(assets.home.rule_left.clone()).with_mode(NodeImageMode::Stretch),
                    ImageNode::new(assets.home.rule_left_highlighted.clone()),
                ),
                (
                    ImageNode::new(assets.home.rule_right.clone())
                        .with_mode(NodeImageMode::Stretch),
                    ImageNode::new(assets.home.rule_right_highlighted.clone()),
                ),
            ],
            help: rule_help_style(assets),
            row_height: R::ROW_HEIGHT,
            value_width: R::VALUE_WIDTH,
        },
    );
}
