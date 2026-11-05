use super::super::input::ConnectionInput;
use super::{ENTRY_DESIGNS, EntryGlow, EntryMotion, GlowKind};
use crate::app::presentation::{MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    ConnectionUiAction, HomeCategory, HomeUiState, InputField, PageTransitionElement, UiAction,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_ui::{CustomButtonMotion, Scrollbar, ScrollbarSkin, UiComposer, spawn_node};

#[derive(Component)]
pub(super) struct EntryArt(pub usize);

#[derive(Component)]
pub(super) struct GameEntry {
    pub index: usize,
    pub art: Entity,
    pub glows: Vec<Entity>,
}

#[derive(Component)]
pub(in crate::app::shell::connection) struct GameLibrary;

#[derive(Component)]
pub(super) struct CategoryUnderline;

pub(in crate::app::shell::connection::view) struct GameGallery<'a> {
    pub(super) assets: &'a UiAssets,
    home: &'a HomeUiState,
    host_port: &'a str,
}

impl<'a> GameGallery<'a> {
    pub fn new(assets: &'a UiAssets, home: &'a HomeUiState, host_port: &'a str) -> Self {
        Self {
            assets,
            home,
            host_port,
        }
    }

    pub fn render(&self, commands: &mut UiComposer, parent: Entity) {
        let gallery = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(gallery)
            .insert((PageTransitionElement::left(0), UiTransform::IDENTITY));
        add_title(commands, gallery, "最近常玩", 23.0, TEXT, self.assets);
        let recent = spawn_node(
            commands,
            gallery,
            Node {
                width: percent(100),
                height: px(180),
                flex_shrink: 0.0,
                column_gap: px(16),
                ..default()
            },
            None,
        );
        for game in self.home.recent() {
            if let Some(index) = ENTRY_DESIGNS
                .iter()
                .position(|entry| entry.game.kind == game)
            {
                self.recent(commands, recent, index);
            }
        }
        let heading = spawn_node(
            commands,
            gallery,
            Node {
                width: percent(100),
                height: px(46),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(24),
                border: UiRect::bottom(px(1)),
                ..default()
            },
            None,
        );
        commands
            .entity(heading)
            .insert(BorderColor::all(Color::srgba(0.6, 0.5, 0.8, 0.35)));
        add_title(commands, heading, "全部游戏", 23.0, TEXT, self.assets);
        let tabs = spawn_node(
            commands,
            heading,
            Node {
                width: px(322),
                height: percent(100),
                position_type: PositionType::Relative,
                column_gap: px(14),
                ..default()
            },
            None,
        );
        for category in HomeCategory::ALL {
            let selected = category == self.home.category;
            let tab = commands.child(
                tabs,
                (
                    Button,
                    CustomButtonMotion,
                    UiAction::Connection(ConnectionUiAction::SelectCategory(category)),
                    Node {
                        width: px(70),
                        flex_shrink: 0.0,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ),
            );
            let label = add_title(
                commands,
                tab,
                category.title(),
                18.0,
                if selected {
                    Color::srgb(0.85, 0.72, 1.0)
                } else {
                    TEXT
                },
                self.assets,
            );
            commands.entity(label).insert(Pickable::IGNORE);
        }
        commands.child(
            tabs,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(14),
                    bottom: px(0),
                    width: px(42),
                    height: px(3),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.82, 0.65, 1.0)),
                CategoryUnderline,
                Pickable::IGNORE,
            ),
        );
        let port = spawn_node(
            commands,
            heading,
            Node {
                margin: UiRect::left(Val::Auto),
                align_items: AlignItems::Center,
                column_gap: px(8),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        add_text(commands, port, "游戏端口 ", 15.0, MUTED, self.assets);
        let input = spawn_node(
            commands,
            port,
            Node {
                width: px(110),
                ..default()
            },
            None,
        );
        ConnectionInput::new(self.host_port, InputField::HostPort, self.assets)
            .compact()
            .render(commands, input);
        let viewport = spawn_node(
            commands,
            gallery,
            Node {
                min_height: px(0),
                height: px(0),
                flex_grow: 1.0,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        let library = spawn_node(
            commands,
            viewport,
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::fr(5, 1.0),
                align_content: AlignContent::Start,
                column_gap: px(18),
                row_gap: px(12),
                padding: UiRect::all(px(8)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(library).insert((
            GameLibrary,
            ScrollPosition(Vec2::ZERO),
            RelativeCursorPosition::default(),
        ));
        Scrollbar::spawn(commands, viewport, library, ScrollbarSkin::Colored(MUTED));
        for (index, entry) in ENTRY_DESIGNS.iter().enumerate() {
            if self.home.category.includes(entry.game.kind) {
                self.entry(commands, library, index);
            }
        }
    }

    pub(super) fn art(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        index: usize,
        size: f32,
    ) -> Entity {
        commands.child(
            parent,
            (
                Node {
                    width: percent(100),
                    max_width: px(size),
                    aspect_ratio: Some(1.0),
                    flex_shrink: 1.0,
                    ..default()
                },
                UiTransform::IDENTITY,
                Pickable::IGNORE,
                EntryArt(index),
                ImageNode::default(),
            ),
        )
    }

    fn entry(&self, commands: &mut UiComposer, parent: Entity, index: usize) {
        let design = &ENTRY_DESIGNS[index];
        let button = commands.keyed_child(
            parent,
            format!("game:{:?}", design.game.kind),
            (
                Button,
                CustomButtonMotion,
                UiAction::Connection(ConnectionUiAction::CreateRoom(design.game.kind)),
                Node {
                    width: percent(100),
                    height: px(156),
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ),
        );
        let stage = spawn_node(
            commands,
            button,
            Node {
                width: percent(100),
                height: px(128),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        commands.entity(stage).insert(Pickable::IGNORE);
        let aura = EntryGlow::spawn(commands, stage, index, GlowKind::Aura);
        let orbit = EntryGlow::spawn(commands, stage, index, GlowKind::Orbit);
        let art = self.art(commands, stage, index, 122.0);
        let caption = spawn_node(
            commands,
            button,
            Node {
                width: percent(100),
                height: px(28),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        commands.entity(caption).insert(Pickable::IGNORE);
        let underline = EntryGlow::spawn(commands, caption, index, GlowKind::Underline);
        let title = add_title(
            commands,
            caption,
            design.game.title,
            19.0,
            TEXT,
            self.assets,
        );
        commands.entity(title).insert(Pickable::IGNORE);
        commands.entity(button).insert((
            EntryMotion::default(),
            GameEntry {
                index,
                art,
                glows: vec![aura, orbit, underline],
            },
        ));
    }
}
