use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, TextLayoutInfo};
use bevy::ui::ComputedUiRenderTargetInfo;

#[derive(Component)]
pub(in super::super) struct GuessName;

pub(in super::super) fn sync_name_widths(
    mut names: Query<(&mut Node, &ComputedTextBlock, &ComputedUiRenderTargetInfo), With<GuessName>>,
) {
    for (mut node, text, target) in &mut names {
        // Use the shaped text's intrinsic width: Bevy's wrapping measure can
        // report a single Chinese character as the entire name's auto width.
        // Flexbox then reduces this width when a long name needs to wrap.
        let width = text.buffer().calculate_content_widths().max;
        if width <= 0.0 {
            continue;
        }
        // Leave one physical pixel for layout rounding at fractional UI scales.
        // Otherwise an exactly fitting last glyph can wrap onto another line.
        let desired = px(((width + 1.0) / target.scale_factor()).ceil());
        if node.width != desired {
            node.width = desired;
        }
    }
}

pub(in super::super) fn sync_name_heights(
    mut names: Query<(&mut Node, Ref<TextLayoutInfo>), With<GuessName>>,
) {
    for (mut node, text) in &mut names {
        if !text.is_changed() || text.size.y <= 0.0 {
            continue;
        }
        // Use the final wrapped text height, so the icon/name row and the
        // player caption reserve separate space and are centered together.
        let desired = px((text.size.y / text.scale_factor).ceil());
        if node.height != desired {
            node.height = desired;
        }
    }
}
