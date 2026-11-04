use crate::app::runtime::UiAssets;
use crate::app::shell::{NavigationUiAction, UiAction, add_page_icon_button};
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(crate) struct ShopEntry<'a> {
    pub assets: &'a UiAssets,
}

impl ShopEntry<'_> {
    pub fn render(&self, commands: &mut UiComposer, parent: Entity, scale: f32) {
        add_page_icon_button(
            commands,
            parent,
            UiAction::Navigation(NavigationUiAction::ToggleShop),
            self.assets,
            self.assets.shop.icon.clone(),
            "商店",
            scale,
        );
    }
}
