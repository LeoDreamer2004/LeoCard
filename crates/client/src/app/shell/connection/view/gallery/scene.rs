use super::super::theme::HOME_SOFT;
use super::{ENTRY_DESIGNS, EntryGlow, GlowKind};
use crate::app::presentation::{CustomButtonMotion, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ConnectionUiAction, PageTransitionElement, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::LayoutConfig;
use bevy::ui_widgets::Button;

#[derive(Component)]
pub(super) struct EntryArt(pub usize);

#[derive(Component)]
pub(super) struct GameEntry {
    pub index: usize,
    pub art: Entity,
    pub glows: [Entity; 4],
}

pub(in crate::app::shell::connection::view) struct GameGallery<'a> {
    assets: &'a UiAssets,
}

impl<'a> GameGallery<'a> {
    pub fn new(assets: &'a UiAssets) -> Self {
        Self { assets }
    }

    pub fn render(&self, commands: &mut Commands, parent: Entity) {
        let gallery = spawn_node(
            commands,
            parent,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.1,
                padding: UiRect::axes(px(12), px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands.entity(gallery).insert((
            PageTransitionElement::left(0),
            UiTransform::IDENTITY,
            LayoutConfig {
                use_rounding: false,
            },
        ));
        for indices in [&[0, 1, 2][..], &[3, 4][..]] {
            let row = spawn_node(
                commands,
                gallery,
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    column_gap: percent(3),
                    ..default()
                },
                None,
            );
            for &index in indices {
                self.entry(commands, row, index);
            }
        }
    }

    fn entry(&self, commands: &mut Commands, parent: Entity, index: usize) {
        let design = &ENTRY_DESIGNS[index];
        let button = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::CreateRoom(design.game.kind)),
                CustomButtonMotion,
                Node {
                    width: percent(31),
                    height: px(246),
                    min_width: px(0),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .id();
        commands.entity(parent).add_child(button);
        let stage = spawn_node(
            commands,
            button,
            Node {
                width: percent(100),
                height: px(190),
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
        let ripple = EntryGlow::spawn(commands, stage, index, GlowKind::Ripple);
        let art = commands
            .spawn((
                Node {
                    width: percent(100),
                    max_width: px(190),
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                EntryArt(index),
                ImageNode::default(),
                UiTransform::IDENTITY,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(stage).add_child(art);
        let caption = spawn_node(
            commands,
            button,
            Node {
                width: percent(100),
                height: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        commands.entity(caption).insert(Pickable::IGNORE);
        let underline = EntryGlow::spawn(commands, caption, index, GlowKind::Underline);
        let title = add_text(
            commands,
            caption,
            design.game.title,
            21.0,
            TEXT,
            self.assets,
        );
        let hint = add_text(
            commands,
            button,
            design.game.description,
            11.0,
            HOME_SOFT.with_alpha(0.75),
            self.assets,
        );
        commands.entity(hint).insert(Node {
            margin: UiRect::top(px(9)),
            ..default()
        });
        for entity in [title, hint] {
            commands.entity(entity).insert(Pickable::IGNORE);
        }
        commands.entity(button).insert(GameEntry {
            index,
            art,
            glows: [aura, orbit, ripple, underline],
        });
    }
}
