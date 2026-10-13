use super::super::MinecraftGuessAssets;
use super::style::{ROW_HEIGHT, SEPARATOR_COLOR, SEPARATOR_WIDTH};
use crate::app::presentation::{UiComposer, spawn_node};
use bevy::{picking::Pickable, prelude::*};

#[derive(Component)]
pub(in super::super) struct GuessEmptyRows {
    guesses: usize,
    height: f32,
}

impl GuessEmptyRows {
    pub(super) fn spawn(commands: &mut UiComposer, scroll: Entity, guesses: usize) {
        let filler = spawn_node(
            commands,
            scroll,
            Node {
                width: percent(100),
                height: px(0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        commands.entity(filler).insert(Self {
            guesses,
            height: -1.0,
        });
    }
}

pub(in super::super) fn fill_empty_rows(
    mut commands: UiComposer,
    assets: Res<MinecraftGuessAssets>,
    viewports: Query<&ComputedNode>,
    mut fillers: Query<(
        Entity,
        &ChildOf,
        &mut GuessEmptyRows,
        &mut Node,
        Option<&Children>,
    )>,
) {
    for (entity, parent, mut filler, mut node, children) in &mut fillers {
        let Ok(viewport) = viewports.get(parent.parent()) else {
            continue;
        };
        let height = (viewport.size().y * viewport.inverse_scale_factor()
            - filler.guesses as f32 * ROW_HEIGHT)
            .max(0.0);
        if (height - filler.height).abs() < 0.5 {
            continue;
        }
        filler.height = height;
        node.height = px(height);
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        let full_rows = (height / ROW_HEIGHT).floor() as usize;
        for _ in 0..full_rows {
            add_blank_row(&mut commands, entity, ROW_HEIGHT, assets.surface(), true);
        }
        let remaining = height - full_rows as f32 * ROW_HEIGHT;
        if remaining > 0.0 {
            add_blank_row(&mut commands, entity, remaining, assets.surface(), false);
        }
    }
}

fn add_blank_row(
    commands: &mut UiComposer,
    parent: Entity,
    height: f32,
    image: ImageNode,
    separator: bool,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(height),
            flex_shrink: 0.0,
            border: UiRect::bottom(px(if separator { SEPARATOR_WIDTH } else { 0.0 })),
            ..default()
        },
        None,
    );
    commands
        .entity(row)
        .insert((image, BorderColor::all(SEPARATOR_COLOR), Pickable::IGNORE));
}
