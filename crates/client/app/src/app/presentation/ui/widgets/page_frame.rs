//! Shared skin for pages and the home screen's framed sections.

use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{PageFrameSkin, sliced_image};

pub(crate) const PAGE_FRAME_TITLE: Color = Color::srgb(0.13, 0.14, 0.17);

pub(crate) fn page_frame_skin(assets: &UiAssets) -> PageFrameSkin {
    // Paint only the title strip; filling the shell behind the inset would
    // reveal its light color when both images fade independently.
    let mut heading =
        ImageNode::new(assets.home.page_frame.clone()).with_mode(NodeImageMode::Stretch);
    heading.rect = Some(Rect::new(60.0, 10.0, 61.0, 11.0));
    PageFrameSkin {
        shell: sliced_image(assets.home.page_frame.clone(), 48.0, 0.5),
        heading,
        inset: sliced_image(assets.home.page_inset.clone(), 48.0, 0.5),
    }
}
