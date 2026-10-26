use super::super::actions::SakuraUiAction;
use super::super::state::SakuraState;
use crate::app::runtime::UiAssets;
use crate::app::shell::{PageTransitionElement, UiAction, add_page_back_title};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct SakuraPage<'a> {
    pub state: &'a SakuraState,
    pub assets: &'a UiAssets,
}

impl SakuraPage<'_> {
    pub fn render(&self, commands: &mut UiComposer, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let page = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1220),
                min_height: px(0),
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        let heading = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(16),
                ..default()
            },
            None,
        );
        commands
            .entity(heading)
            .insert((PageTransitionElement::left(0), UiTransform::IDENTITY));
        add_page_back_title(
            commands,
            heading,
            "SakuraFrp 内网穿透",
            UiAction::Sakura(SakuraUiAction::Close),
            self.assets,
        );
        if self.state.client.is_none() {
            self.render_login(commands, page);
            return;
        }
        self.render_account(commands, heading);
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                column_gap: px(18),
                ..default()
            },
            None,
        );
        self.render_tunnels(commands, body);
        self.render_form(commands, root, body);
    }
}
