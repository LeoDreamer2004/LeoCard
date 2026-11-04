use super::ShopUiState;
#[cfg(feature = "developer")]
use super::developer::add_coin_input;
use super::items::{ITEM_CARD_WIDTH, render_item};
use super::skin::add_categories;
use crate::app::presentation::{MUTED, add_coin_balance, add_title, page_frame_skin};
use crate::app::runtime::UiAssets;
use crate::app::shell::{NavigationUiAction, PageTransitionElement, UiAction, add_page_back_title};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_ui::{PageFrame, Scrollbar, ScrollbarSkin, UiComposer, spawn_node};

#[derive(Component)]
pub(super) struct ShopList;

pub(crate) struct ShopPage<'a> {
    pub assets: &'a UiAssets,
    pub economy: &'a PlayerEconomy,
    pub state: &'a ShopUiState,
    pub in_game: bool,
}

impl ShopPage<'_> {
    pub fn render(&self, commands: &mut UiComposer, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                margin: UiRect::top(px(if self.in_game { 72.0 } else { 0.0 })),
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
                max_width: px(1320),
                padding: UiRect::all(px(if cfg!(target_os = "android") { 12 } else { 20 })),
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
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
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
            "商店",
            UiAction::Navigation(NavigationUiAction::ToggleShop),
            self.assets,
        );
        let wallet = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        #[cfg(feature = "developer")]
        add_coin_input(commands, wallet, self.economy.coins(), self.assets);
        add_coin_balance(
            commands,
            wallet,
            self.economy.coins(),
            27.0,
            19.0,
            self.assets,
        );
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        add_categories(commands, body, self.assets, self.state);
        let mut frame = PageFrame::new(page_frame_skin(self.assets));
        frame.node.flex_basis = px(0);
        frame.node.flex_grow = 1.0;
        frame.node.margin = UiRect::left(px(-8));
        let frame = frame.mount(commands, body);
        commands
            .entity(frame.panel)
            .insert((PageTransitionElement::right(2), UiTransform::IDENTITY));
        add_title(
            commands,
            frame.heading,
            self.state.category.title(),
            21.0,
            Color::srgb(0.13, 0.14, 0.17),
            self.assets,
        );
        let viewport = spawn_node(
            commands,
            frame.content,
            Node {
                height: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        let list = spawn_node(
            commands,
            viewport,
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(
                    GridTrackRepetition::AutoFit,
                    ITEM_CARD_WIDTH,
                ),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                row_gap: px(18),
                column_gap: px(18),
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(4)),
                ..default()
            },
            None,
        );
        commands.entity(list).insert((
            ShopList,
            ScrollPosition(Vec2::ZERO),
            RelativeCursorPosition::default(),
        ));
        Scrollbar::spawn(commands, viewport, list, ScrollbarSkin::Colored(MUTED));
        for item in ItemId::ALL {
            render_item(commands, list, item, self.economy, self.assets);
        }
    }
}
