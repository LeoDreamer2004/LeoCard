use crate::app::presentation::TEXT;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{HelpTooltipStyle, UiComposer, add_help_tooltip, sliced_image};

pub(super) fn rule_help_style(assets: &UiAssets) -> HelpTooltipStyle<'_> {
    HelpTooltipStyle {
        font: &assets.font,
        icon: ImageNode::new(assets.home.help_question.clone()),
        background: sliced_image(assets.home.input.clone(), 32.0, 0.55),
        text: TEXT,
        width: 290.0,
    }
}

pub(crate) fn add_rule_help(
    commands: &mut UiComposer,
    parent: Entity,
    help: &str,
    assets: &UiAssets,
) {
    add_help_tooltip(commands, parent, help, rule_help_style(assets));
}
