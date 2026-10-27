use crate::app::presentation::TEXT;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{HelpTooltipStyle, sliced_image};

pub(crate) fn rule_help_style(assets: &UiAssets) -> HelpTooltipStyle<'_> {
    HelpTooltipStyle {
        font: &assets.font,
        icon: ImageNode::new(assets.home.help_question.clone()),
        background: sliced_image(assets.home.input.clone(), 32.0, 0.55),
        text: TEXT,
        width: 290.0,
    }
}
