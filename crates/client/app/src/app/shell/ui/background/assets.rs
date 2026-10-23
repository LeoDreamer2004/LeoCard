//! Resources owned by the shared backdrop, independent of page UI assets.

use super::BackgroundMaterial;
use bevy::prelude::*;

const SUITS_ATLAS: &str = "ui/background/suits-atlas.png";
const ATLAS_CELL: f32 = 627.0;

#[derive(Resource)]
pub(super) struct BackgroundAssets {
    pub material: Handle<BackgroundMaterial>,
    suits: Handle<Image>,
}

impl FromWorld for BackgroundAssets {
    fn from_world(world: &mut World) -> Self {
        let suits = world.resource::<AssetServer>().load(SUITS_ATLAS);
        let material = world
            .resource_mut::<Assets<BackgroundMaterial>>()
            .add(BackgroundMaterial::default());
        Self { material, suits }
    }
}

impl BackgroundAssets {
    pub fn suit(&self, index: usize) -> ImageNode {
        let origin = Vec2::new((index % 2) as f32, (index / 2) as f32) * ATLAS_CELL;
        ImageNode::new(self.suits.clone())
            .with_rect(Rect::from_corners(origin, origin + Vec2::splat(ATLAS_CELL)))
            .with_mode(NodeImageMode::Stretch)
    }
}
