use super::{Scrollbar, ScrollbarSkin};
use crate::app::presentation::{UiComposer, spawn_node};
use bevy::{picking::Pickable, prelude::*, ui::RelativeCursorPosition};

impl Scrollbar {
    pub(crate) fn spawn(
        commands: &mut UiComposer,
        parent: Entity,
        scroll: Entity,
        skin: ScrollbarSkin,
    ) {
        let textured = matches!(skin, ScrollbarSkin::Textured(_));
        let track = spawn_node(
            commands,
            parent,
            Node {
                width: px(if textured { 18.0 } else { 12.0 }),
                height: percent(100),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(px(if textured { 0.0 } else { 6.0 })),
                ..default()
            },
            None,
        );
        let thumb = spawn_node(
            commands,
            track,
            Node {
                position_type: PositionType::Absolute,
                width: if textured { Val::Auto } else { percent(100) },
                left: px(if textured { 2.0 } else { 0.0 }),
                right: if textured { px(2) } else { Val::Auto },
                height: px(32),
                border_radius: BorderRadius::all(px(if textured { 0.0 } else { 6.0 })),
                ..default()
            },
            None,
        );
        match &skin {
            ScrollbarSkin::Colored(color) => {
                commands
                    .entity(track)
                    .insert(BackgroundColor(color.with_alpha(0.12)));
                commands
                    .entity(thumb)
                    .insert(BackgroundColor(color.with_alpha(0.7)));
            }
            ScrollbarSkin::Textured(images) => {
                commands.entity(track).insert(images.track.clone());
                commands.entity(thumb).insert(images.thumb.clone());
            }
        }
        commands.entity(thumb).insert(Pickable::IGNORE);
        commands.entity(track).insert((
            Self {
                scroll,
                thumb,
                skin,
            },
            RelativeCursorPosition::default(),
        ));
    }
}
