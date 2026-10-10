#[cfg(feature = "developer")]
use super::developer::add_coin_input;
use super::{ShopUiAction, ShopUiState};
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{
    ACCENT, MUTED, PanelSkin, TEXT, add_coin_balance, add_text, decorate_panel_skin, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, NavigationUiAction, PageTransitionElement, UiAction,
    add_cozy_button_variant, add_cozy_disabled_button, add_page_back_title,
};
use bevy::picking::Pickable;
use bevy::{prelude::*, ui::RelativeCursorPosition};
use leocard_client::{ItemId, PlayerEconomy};

#[derive(Component)]
pub(super) struct ShopList;

#[derive(Component)]
pub(super) struct ItemRemaining {
    pub item: ItemId,
    pub active: bool,
}

pub(crate) struct ShopPage<'a> {
    pub assets: &'a UiAssets,
    pub economy: &'a PlayerEconomy,
    pub state: &'a ShopUiState,
    pub in_game: bool,
}

impl ShopPage<'_> {
    pub fn render(&self, commands: &mut Commands, root: Entity) {
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
                max_width: px(1180),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(20),
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
                column_gap: px(20),
                ..default()
            },
            None,
        );
        let categories = spawn_node(
            commands,
            body,
            Node {
                width: px(160),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(categories)
            .insert((PageTransitionElement::left(1), UiTransform::IDENTITY));
        for game in SUPPORTED_GAMES {
            add_cozy_button_variant(
                commands,
                categories,
                game.title,
                UiAction::Shop(ShopUiAction::SelectCategory(game.kind)),
                self.assets,
                percent(100),
                48.0,
                if self.state.category == game.kind {
                    CozyButtonVariant::Primary
                } else {
                    CozyButtonVariant::Neutral
                },
            );
        }
        let list = spawn_node(
            commands,
            body,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(list).insert((
            PageTransitionElement::right(2),
            UiTransform::IDENTITY,
            ShopList,
            ScrollPosition(Vec2::ZERO),
            RelativeCursorPosition::default(),
        ));
        let mut empty = true;
        for item in ItemId::ALL {
            if item.definition().game == self.state.category {
                self.render_item(commands, list, item);
                empty = false;
            }
        }
        if empty {
            add_text(commands, list, "暂无道具", 18.0, MUTED, self.assets);
        }
    }

    fn render_item(&self, commands: &mut Commands, parent: Entity, id: ItemId) {
        let item = id.definition();
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                padding: UiRect::axes(px(22), px(20)),
                align_items: AlignItems::Center,
                column_gap: px(20),
                ..default()
            },
            None,
        );
        decorate_panel_skin(commands, row, PanelSkin::Section, self.assets);
        let identity = spawn_node(
            commands,
            row,
            Node {
                width: px(110),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        self.icon(
            commands,
            identity,
            match id {
                ItemId::ObservationLens => self.assets.shop.observation_lens.clone(),
                ItemId::ShengjiCardCounter | ItemId::QiGui523CardCounter => {
                    self.assets.shop.card_counter.clone()
                }
                ItemId::ShengjiMissingSuitCard => self.assets.shop.missing_suit_card.clone(),
                ItemId::UnoJumpInDevice => self.assets.shop.uno_jump_in_device.clone(),
                ItemId::MahjongFanCalculator => self.assets.shop.mahjong_fan_calculator.clone(),
            },
            64.0,
        );
        add_text(commands, identity, item.name, 21.0, TEXT, self.assets);
        let description = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        add_text(
            commands,
            description,
            item.description,
            16.0,
            MUTED,
            self.assets,
        );
        let purchase = spawn_node(
            commands,
            row,
            Node {
                width: px(160),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        let active = self.economy.active(id);
        let label = format!("{} {}", if active { "延期" } else { "购买" }, item.price);
        let button = if self.economy.error().is_some() || self.economy.coins() < item.price {
            add_cozy_disabled_button(commands, purchase, &label, self.assets, percent(100), 44.0)
        } else {
            add_cozy_button_variant(
                commands,
                purchase,
                &label,
                UiAction::Shop(ShopUiAction::Buy(id)),
                self.assets,
                percent(100),
                44.0,
                CozyButtonVariant::Cool,
            )
        };
        commands.entity(button).insert(Node {
            width: percent(100),
            height: px(44),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(5),
            ..default()
        });
        self.icon(commands, button, self.assets.shop.coin.clone(), 22.0);
        let remaining = add_text(
            commands,
            purchase,
            remaining_label(self.economy, id),
            14.0,
            if active { ACCENT } else { MUTED },
            self.assets,
        );
        commands
            .entity(remaining)
            .insert(ItemRemaining { item: id, active });
    }

    fn icon(&self, commands: &mut Commands, parent: Entity, image: Handle<Image>, size: f32) {
        let icon = commands
            .spawn((
                Node {
                    width: px(size),
                    height: px(size),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(image),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(parent).add_child(icon);
    }
}

pub(super) fn remaining_label(economy: &PlayerEconomy, item: ItemId) -> String {
    let remaining = economy.remaining_seconds(item);
    if remaining > 0 {
        format!(
            "剩余 {:02}:{:02}:{:02}",
            remaining / 3600,
            remaining / 60 % 60,
            remaining % 60
        )
    } else if economy.error().is_some() {
        "钱包暂不可用".to_owned()
    } else if economy.coins() < item.definition().price {
        "金币不足".to_owned()
    } else {
        "有效期一小时".to_owned()
    }
}
