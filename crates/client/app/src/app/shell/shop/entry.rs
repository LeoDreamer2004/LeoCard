use crate::app::presentation::UiComposer;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, NavigationUiAction, UiAction, add_cozy_labeled_icon_button,
};
use bevy::prelude::*;

pub(crate) struct ShopEntry<'a> {
    pub assets: &'a UiAssets,
}

impl ShopEntry<'_> {
    pub fn render(&self, commands: &mut UiComposer, parent: Entity) {
        add_cozy_labeled_icon_button(
            commands,
            parent,
            UiAction::Navigation(NavigationUiAction::ToggleShop),
            self.assets,
            self.assets.shop.icon.clone(),
            CozyButtonVariant::Neutral,
            "商店",
        );
    }
}
