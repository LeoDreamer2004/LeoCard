use super::ShopUiAction;
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, add_title, rule_help_style};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_ui::{UiComposer, add_help_tooltip, decorate_panel, sliced_image, spawn_node};

pub(super) const ITEM_CARD_WIDTH: f32 = 216.0;

#[derive(Component)]
pub(super) struct ItemRemaining {
    pub item: ItemId,
    pub active: bool,
}

pub(super) fn render_item(
    commands: &mut UiComposer,
    parent: Entity,
    id: ItemId,
    economy: &PlayerEconomy,
    assets: &UiAssets,
) {
    let item = id.definition();
    let card = spawn_node(
        commands,
        parent,
        Node {
            width: px(ITEM_CARD_WIDTH),
            max_width: percent(100),
            flex_shrink: 0.0,
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(10),
            ..default()
        },
        None,
    );
    decorate_panel(
        commands,
        card,
        sliced_image(assets.shop.item_frame.clone(), 24.0, 0.5),
    );
    let header = spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            height: px(58),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            border: UiRect::bottom(px(1)),
            ..default()
        },
        None,
    );
    commands
        .entity(header)
        .insert(BorderColor::all(Color::srgba(0.7, 0.72, 0.8, 0.18)));
    let identity = spawn_node(
        commands,
        header,
        Node {
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    add_title(commands, identity, item.name, 21.0, TEXT, assets);
    let game = SUPPORTED_GAMES
        .iter()
        .find(|game| game.kind == item.game)
        .unwrap();
    add_text(commands, identity, game.title, 13.0, MUTED, assets);
    add_help_tooltip(commands, header, item.description, rule_help_style(assets));
    let artwork = spawn_node(
        commands,
        card,
        Node {
            width: px(144),
            height: px(144),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    decorate_panel(
        commands,
        artwork,
        sliced_image(assets.home.input.clone(), 32.0, 0.4),
    );
    icon(
        commands,
        artwork,
        match id {
            ItemId::ObservationLens => assets.shop.observation_lens.clone(),
            ItemId::ShengjiCardCounter
            | ItemId::QiGui523CardCounter
            | ItemId::DoudizhuCardCounter => assets.shop.card_counter.clone(),
            ItemId::ShengjiMissingSuitCard => assets.shop.missing_suit_card.clone(),
            ItemId::UnoJumpInDevice => assets.shop.uno_jump_in_device.clone(),
            ItemId::MahjongFanCalculator => assets.shop.mahjong_fan_calculator.clone(),
        },
        120.0,
    );
    let active = economy.active(id);
    let remaining = add_text(
        commands,
        card,
        remaining_label(economy, id),
        14.0,
        if active { ACCENT } else { MUTED },
        assets,
    );
    commands
        .entity(remaining)
        .insert(ItemRemaining { item: id, active });
    let label = if economy.error().is_some() {
        "钱包暂不可用"
    } else if economy.coins() < item.price {
        "金币不足"
    } else if active {
        "延期"
    } else {
        "购买"
    };
    let button = if economy.error().is_some() || economy.coins() < item.price {
        add_cozy_disabled_button(commands, card, label, assets, percent(100), 44.0)
    } else {
        add_cozy_button_variant(
            commands,
            card,
            label,
            UiAction::Shop(ShopUiAction::Buy(id)),
            assets,
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
    icon(commands, button, assets.shop.coin.clone(), 21.0);
    add_text(commands, button, item.price.to_string(), 17.0, TEXT, assets);
}

fn icon(commands: &mut UiComposer, parent: Entity, image: Handle<Image>, size: f32) {
    commands.child(
        parent,
        (
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(image),
            Pickable::IGNORE,
        ),
    );
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
    } else {
        "生效1小时".to_owned()
    }
}
