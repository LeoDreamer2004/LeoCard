//! A persistent, non-interactive backdrop shared by all opted-in pages.

use super::{BackgroundAssets, FloatingSuit, SUITS};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::LayoutConfig;

#[derive(Component)]
pub(super) struct PageBackground;

#[derive(Component)]
pub(super) struct BackgroundCanvas;

/// Opt a page into the shared backdrop. The backdrop itself survives UI rebuilds.
pub(crate) fn add_page_background(commands: &mut Commands, root: Entity) {
    commands.entity(root).insert(PageBackground);
}

pub(super) fn setup_background(
    mut commands: Commands,
    assets: Res<BackgroundAssets>,
    time: Res<Time>,
) {
    let canvas = commands
        .spawn((
            BackgroundCanvas,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            MaterialNode(assets.material.clone()),
            // Slow artwork motion must not snap to the physical pixel grid.
            LayoutConfig {
                use_rounding: false,
            },
            GlobalZIndex(-100),
            Pickable::IGNORE,
            Visibility::Hidden,
        ))
        .id();
    for (index, placement) in SUITS.into_iter().enumerate() {
        let suit = FloatingSuit::new(placement, index);
        let (transform, color) = suit.sample(time.elapsed_secs());
        let decoration = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(placement.center.x),
                    top: percent(placement.center.y),
                    width: Val::VMin(placement.size),
                    height: Val::VMin(placement.size),
                    margin: UiRect {
                        left: Val::VMin(-placement.size * 0.5),
                        top: Val::VMin(-placement.size * 0.5),
                        ..default()
                    },
                    ..default()
                },
                assets.suit(placement.atlas).with_color(color),
                transform,
                suit,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(canvas).add_child(decoration);
    }
}

pub(super) fn sync_background_visibility(
    pages: Query<(), With<PageBackground>>,
    mut canvases: Query<&mut Visibility, With<BackgroundCanvas>>,
) {
    let visibility = if pages.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    for mut canvas in &mut canvases {
        if *canvas != visibility {
            *canvas = visibility;
        }
    }
}
