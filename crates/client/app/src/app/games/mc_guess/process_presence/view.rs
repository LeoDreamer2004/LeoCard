use crate::app::presentation::{DANGER, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{UiComposer, add_avatar_status_overlay};

pub(in super::super) fn add_process_presence_overlay(
    commands: &mut UiComposer,
    avatar: Entity,
    size: f32,
    assets: &UiAssets,
) {
    let overlay = add_avatar_status_overlay(commands, avatar, size, DANGER);
    let symbol = add_text(commands, overlay, "?", size * 0.75, TEXT, assets);
    commands.entity(symbol).insert((
        TextShadow::default(),
        TextLayout::default().with_no_wrap(),
        Pickable::IGNORE,
    ));
}
