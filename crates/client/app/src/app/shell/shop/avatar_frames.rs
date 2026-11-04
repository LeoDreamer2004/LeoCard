use super::ShopUiAction;
use super::skin::{add_price, add_product_card, add_product_preview};
use crate::app::presentation::{TEXT, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
};
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, ShopAvatarFrame};
use leocard_ui::UiComposer;

pub(super) fn render_frame(
    commands: &mut UiComposer,
    parent: Entity,
    frame: ShopAvatarFrame,
    economy: &PlayerEconomy,
    assets: &UiAssets,
) {
    let card = add_product_card(commands, parent, assets);
    add_title(commands, card, frame.name, 21.0, TEXT, assets);
    let preview = add_product_preview(commands, card, assets, frame.rarity);
    assets
        .avatar_frames
        .frame(frame.id, 120.0)
        .mount_preview(commands, preview);

    let owned = economy.owns_frame(frame.id);
    let exclusive = economy.is_exclusive_frame(frame.id);
    let equipped = economy.equipped_frame() == frame.id;
    let (label, action) = if exclusive {
        (
            "免费使用",
            (!equipped).then_some(ShopUiAction::EquipFrame(frame.id)),
        )
    } else if equipped {
        ("使用中", None)
    } else if economy.error().is_some() {
        ("钱包暂不可用", None)
    } else if owned {
        ("使用", Some(ShopUiAction::EquipFrame(frame.id)))
    } else if economy.coins() < frame.price {
        ("金币不足", None)
    } else {
        ("购买", Some(ShopUiAction::BuyFrame(frame.id)))
    };
    let button = match action {
        Some(action) => add_cozy_button_variant(
            commands,
            card,
            label,
            UiAction::Shop(action),
            assets,
            percent(100),
            44.0,
            CozyButtonVariant::Cool,
        ),
        None => add_cozy_disabled_button(commands, card, label, assets, percent(100), 44.0),
    };
    if !owned && !exclusive {
        add_price(commands, button, frame.price, assets);
    }
}
