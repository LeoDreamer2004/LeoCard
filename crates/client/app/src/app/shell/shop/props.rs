use super::ShopUiAction;
use super::skin::{add_price, add_product_card, add_product_icon, add_product_preview};
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, add_title, rule_help_style};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
    add_egg_thrower_icon,
};
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};
use leocard_ui::{UiComposer, add_help_tooltip, spawn_node};

#[derive(Component)]
pub(super) struct PropRemaining {
    pub prop: PropId,
    pub active: bool,
}

pub(super) fn render_prop(
    commands: &mut UiComposer,
    parent: Entity,
    id: PropId,
    economy: &PlayerEconomy,
    assets: &UiAssets,
) {
    let prop = id.definition();
    let card = add_product_card(commands, parent, assets);
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
    add_title(commands, identity, prop.name, 21.0, TEXT, assets);
    add_text(commands, identity, prop.game_label, 13.0, MUTED, assets);
    add_help_tooltip(commands, header, prop.description, rule_help_style(assets));
    let artwork = add_product_preview(commands, card, assets, prop.rarity);
    if id == PropId::AutoEggThrower {
        add_egg_thrower_icon(commands, artwork, 120.0, assets);
    } else {
        let image = match id {
            PropId::ObservationLens => assets.props.observation_lens.clone(),
            PropId::CardCounter
            | PropId::ShengjiCardCounter
            | PropId::QiGui523CardCounter
            | PropId::DoudizhuCardCounter => assets.props.card_counter.clone(),
            PropId::ShengjiMissingSuitCard => assets.props.missing_suit_card.clone(),
            PropId::UnoJumpInDevice => assets.props.uno_jump_in_device.clone(),
            PropId::MahjongFanCalculator => assets.props.mahjong_fan_calculator.clone(),
            PropId::AutoEggThrower => unreachable!("consumable artwork is composed above"),
        };
        add_product_icon(commands, artwork, image, 120.0);
    }
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
        .insert(PropRemaining { prop: id, active });
    let label = if economy.error().is_some() {
        "钱包暂不可用"
    } else if economy.coins() < prop.price {
        "金币不足"
    } else if active {
        "延期"
    } else {
        "购买"
    };
    let button = if economy.error().is_some() || economy.coins() < prop.price {
        add_cozy_disabled_button(commands, card, label, assets, percent(100), 44.0)
    } else {
        add_cozy_button_variant(
            commands,
            card,
            label,
            UiAction::Shop(ShopUiAction::BuyProp(id)),
            assets,
            percent(100),
            44.0,
            CozyButtonVariant::Cool,
        )
    };
    add_price(commands, button, prop.price, assets);
}

pub(super) fn remaining_label(economy: &PlayerEconomy, prop: PropId) -> String {
    if prop.is_consumable() {
        return format!("已拥有 {} 个", economy.prop_count(prop));
    }
    let remaining = economy.remaining_seconds(prop);
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
