//! A full-window card table, independent of the game's fixed-width playing area.

use crate::app::presentation::{TableBackground, TableBackgroundMaterial, TableSurfaceKind};
use crate::app::runtime::TableAssets;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(crate) struct CardTableSurface<'a> {
    pub brightness: f32,
    pub default_felt: &'a Handle<Image>,
    pub custom_felt: Option<&'a Handle<Image>>,
}

impl CardTableSurface<'_> {
    pub(crate) fn apply(
        self,
        commands: &mut UiComposer,
        surface: Entity,
        tables: &TableAssets,
        materials: &mut Assets<TableBackgroundMaterial>,
    ) {
        let material = commands.material(
            surface,
            materials,
            TableBackgroundMaterial::new(
                TableSurfaceKind::Cards,
                self.brightness,
                tables,
                self.default_felt,
                self.custom_felt,
            ),
        );
        commands
            .entity(surface)
            .insert((MaterialNode(material), TableBackground));
    }
}
