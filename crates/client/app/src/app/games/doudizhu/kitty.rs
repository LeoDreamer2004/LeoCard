use super::cards::card_row;
use super::{DoudizhuTable, DoudizhuUiAction};
use crate::app::presentation::{
    ACCENT, CardSize, HEADER_BG, PanelSkin, add_text, decorate_panel_skin,
};
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{UiComposer, spawn_node};

impl DoudizhuTable<'_> {
    pub(super) fn kitty(&self, commands: &mut UiComposer, parent: Entity) {
        if !self.ui.kitty_open || self.game.kitty.is_empty() {
            return;
        }
        let backdrop = spawn_node(
            commands,
            parent,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::BLACK.with_alpha(0.48)),
        );
        commands.entity(backdrop).insert((
            Button,
            UiAction::Doudizhu(DoudizhuUiAction::ToggleKitty),
            GlobalZIndex(1750),
        ));
        let panel = spawn_node(
            commands,
            backdrop,
            Node {
                min_width: px(430),
                min_height: px(184),
                padding: UiRect::axes(px(28), px(22)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(11),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.98)),
        );
        commands.entity(panel).insert(Pickable::IGNORE);
        decorate_panel_skin(commands, panel, PanelSkin::Popup, self.assets);
        add_text(commands, panel, "底牌", 22.0, ACCENT, self.assets);
        card_row(
            commands,
            panel,
            &self.game.kitty,
            CardSize::TrickSeat,
            self.assets,
        );
    }
}
