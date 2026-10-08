use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{CozyButtonVariant, NavigationUiAction, UiAction, add_cozy_icon_button};
use bevy::prelude::*;

pub(crate) struct ShopEntry<'a> {
    pub assets: &'a UiAssets,
}

impl ShopEntry<'_> {
    pub fn render(&self, commands: &mut Commands, parent: Entity) {
        let column = spawn_node(
            commands,
            parent,
            Node {
                width: px(44),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(1),
                ..default()
            },
            None,
        );
        add_cozy_icon_button(
            commands,
            column,
            UiAction::Navigation(NavigationUiAction::ToggleShop),
            self.assets,
            self.assets.shop.icon.clone(),
            CozyButtonVariant::Neutral,
        );
        add_text(commands, column, "商店", 12.0, TEXT, self.assets);
    }
}
