use super::scene::GameGallery;
use super::{ENTRY_DESIGNS, EntryMotion, GameEntry};
use crate::app::presentation::{TEXT, add_text, add_title};
use crate::app::shell::{ConnectionUiAction, LiquidKind, LiquidSurface, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{CustomButtonMotion, UiComposer, spawn_node};

impl GameGallery<'_> {
    pub(super) fn recent(&self, commands: &mut UiComposer, parent: Entity, index: usize) {
        let design = &ENTRY_DESIGNS[index];
        let button = commands.keyed_child(
            parent,
            format!("game:{:?}", design.game.kind),
            (
                Button,
                CustomButtonMotion,
                UiAction::Connection(ConnectionUiAction::CreateRoom(design.game.kind)),
                Node {
                    flex_grow: 1.0,
                    flex_basis: px(0),
                    min_width: px(0),
                    height: percent(100),
                    padding: UiRect::axes(px(22), px(12)),
                    column_gap: px(14),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ),
        );
        LiquidSurface::mount(
            commands,
            button,
            button,
            self.assets.home.recent_cards[design.surface as usize].clone(),
            design.accent,
            LiquidKind::Card,
            index as f32 * 1.37,
        );
        let art = self.art(commands, button, index, 144.0);
        commands.entity(art).insert(Node {
            width: px(144),
            height: px(144),
            flex_shrink: 0.0,
            ..default()
        });
        let caption = spawn_node(
            commands,
            button,
            Node {
                width: px(182),
                min_width: px(0),
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        let title = add_title(
            commands,
            caption,
            design.game.title,
            26.0,
            TEXT,
            self.assets,
        );
        commands.entity(title).insert(Pickable::IGNORE);
        let subtitle = add_text(
            commands,
            caption,
            design.game.description,
            13.0,
            Color::srgb(0.79, 0.80, 0.75),
            self.assets,
        );
        commands
            .entity(subtitle)
            .insert((Pickable::IGNORE, TextLayout::default().with_no_wrap()));
        let play = commands.child(
            caption,
            (
                Node {
                    width: px(160),
                    max_width: percent(100),
                    height: px(48),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
        let surface = LiquidSurface::mount(
            commands,
            play,
            button,
            self.assets.home.play_table.clone(),
            design.accent,
            LiquidKind::Accent,
            0.0,
        );
        LiquidSurface::set_rect(
            commands,
            surface,
            Vec4::new(200.0 / 2172.0, 95.0 / 724.0, 1970.0 / 2172.0, 650.0 / 724.0),
        );
        commands.entity(play).insert(Pickable::IGNORE);
        commands.entity(caption).insert(Pickable::IGNORE);
        commands.entity(button).insert((
            EntryMotion::default(),
            GameEntry {
                index,
                art,
                glows: Vec::new(),
            },
        ));
    }
}
