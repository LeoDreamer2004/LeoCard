use crate::app::runtime::PageErrorState;
use crate::app::shell::{
    ConfirmationDialog, DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState,
    dispatch_domain_actions,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::GameKind;

#[derive(Clone)]
pub(crate) enum ShopUiAction {
    SelectCategory(GameKind),
    Buy(ItemId),
    ConfirmPurchase { item: ItemId, extend: bool },
}

impl DomainUiAction for ShopUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Shop(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct ShopActionContext<'w> {
    economy: ResMut<'w, PlayerEconomy>,
    confirmation: ResMut<'w, ConfirmationDialog>,
    ui: ResMut<'w, UiState>,
    error: ResMut<'w, PageErrorState>,
}

pub(super) fn dispatch_shop_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ShopActionContext,
) {
    dispatch_domain_actions::<ShopUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ShopActionContext<'_>> for ShopUiAction {
    fn handle(&self, context: &mut ShopActionContext<'_>) {
        match self {
            Self::SelectCategory(category) => context.ui.shop.category = *category,
            Self::Buy(item) => {
                let definition = item.definition();
                let extend = context.economy.active(*item);
                let verb = if extend { "延期" } else { "购买" };
                let detail = if extend {
                    "确认后在原截止时间上追加一小时。"
                } else {
                    "购买后立即生效，有效期一小时。"
                };
                context.confirmation.show(
                    format!("确认{verb}"),
                    format!(
                        "花费 {} 金币{verb}{}？\n{detail}",
                        definition.price, definition.name
                    ),
                    UiAction::Shop(Self::ConfirmPurchase {
                        item: *item,
                        extend,
                    }),
                );
            }
            Self::ConfirmPurchase { item, extend } => {
                context.ui.dirty = true;
                let result = if *extend {
                    context.economy.extend(*item)
                } else {
                    context.economy.buy(*item)
                };
                if let Err(error) = result {
                    context.error.error = Some(error);
                }
            }
        }
    }
}
