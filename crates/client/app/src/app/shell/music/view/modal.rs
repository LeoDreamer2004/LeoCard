use super::super::{MusicMotion, MusicState};
use super::panel::render_music;
use crate::app::presentation::{PAGE_FRAME_TITLE, add_title, page_frame_skin};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction, UiAction,
    add_configuration_scroll, add_cozy_close_button, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{PageFrame, UiComposer, spawn_node};

pub(crate) fn render_music_modal(
    commands: &mut UiComposer,
    root: Entity,
    state: &MusicState,
    motion: &MusicMotion,
    assets: &UiAssets,
) {
    if !motion.visible {
        return;
    }
    let overlay = commands.keyed_child(
        root,
        "music-modal",
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(cozy_backdrop_color(motion.progress)),
            GlobalZIndex(2050),
            Pickable::default(),
            CozyModalBackdrop(CozyModalKind::Music),
        ),
    );
    let mut frame = PageFrame::new(page_frame_skin(assets));
    frame.node.width = px(760);
    frame.node.height = px(640);
    frame.node.max_width = percent(92);
    frame.node.max_height = percent(92);
    frame.heading.height = px(48);
    let frame = frame.mount(commands, overlay);
    commands.entity(frame.panel).insert((
        CozyModalPanel(CozyModalKind::Music),
        cozy_panel_transform(motion.progress),
    ));
    add_title(
        commands,
        frame.heading,
        "音乐播放器",
        28.0,
        PAGE_FRAME_TITLE,
        assets,
    );
    add_cozy_close_button(
        commands,
        frame.heading,
        UiAction::Navigation(NavigationUiAction::ToggleMusic),
        assets,
    );
    let content = spawn_node(
        commands,
        frame.content,
        Node {
            min_width: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            padding: UiRect::all(px(18)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    let content = add_configuration_scroll(commands, content, "music-library", 12.0);
    render_music(commands, content, state, assets);
}
