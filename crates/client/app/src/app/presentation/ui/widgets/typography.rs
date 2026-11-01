//! Shared body and heading typography, including fallback for title glyphs.

use super::super::ACCENT;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::text::FontSource;
use leocard_ui::{UiComposer, label_scene, ui_component};

pub(crate) fn add_section_title(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    assets: &UiAssets,
) {
    add_title(commands, parent, text, 21.0, ACCENT, assets);
}

#[track_caller]
pub(crate) fn add_title(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    commands.node(parent, title_scene(assets, text, size, color))
}

pub(crate) fn title_scene(
    assets: &UiAssets,
    text: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Scene {
    let text = Text::new(text);
    let font = title_font(assets, size);
    bsn! {
        text
        @ui_component(font)
        TextColor(color)
    }
}

pub(crate) fn title_font(assets: &UiAssets, size: f32) -> TextFont {
    TextFont {
        font: FontSource::list([assets.title_font.clone(), assets.font.clone()]),
        ..TextFont::from_font_size(size)
    }
}

#[track_caller]
pub(crate) fn add_text(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    commands.node(parent, text_scene(assets, text, size, color))
}

pub(crate) fn text_scene(
    assets: &UiAssets,
    text: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Scene {
    label_scene(assets.font.clone(), text, size, color)
}
