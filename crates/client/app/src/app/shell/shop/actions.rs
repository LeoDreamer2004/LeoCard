use super::ShopCategory;
use crate::app::achievements::LocalAchievementTrigger;
use crate::app::runtime::PageErrorState;
use crate::app::shell::{
    ConfirmationDialog, DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState,
    dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_client::{PlayerEconomy, PropId, ShopAvatarFrame};
use leocard_protocol::AvatarFrameId;

#[derive(Clone)]
pub(crate) enum ShopUiAction {
    SelectCategory(ShopCategory),
    BuyProp(PropId),
    ConfirmPropPurchase { prop: PropId, extend: bool },
    BuyFrame(AvatarFrameId),
    ConfirmFramePurchase(AvatarFrameId),
    EquipFrame(AvatarFrameId),
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
    achievements: MessageWriter<'w, LocalAchievementTrigger>,
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
            Self::BuyProp(prop) => {
                let definition = prop.definition();
                let extend = context.economy.active(*prop);
                let verb = if extend { "延期" } else { "购买" };
                let detail = if prop.is_consumable() {
                    "购买后数量增加1个，使用时消耗，不会过期。"
                } else if extend {
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
                    UiAction::Shop(Self::ConfirmPropPurchase {
                        prop: *prop,
                        extend,
                    }),
                );
            }
            Self::ConfirmPropPurchase { prop, extend } => {
                context.ui.dirty = true;
                let result = if *extend {
                    context.economy.extend_prop(*prop)
                } else {
                    context.economy.buy_prop(*prop)
                };
                match result {
                    Ok(()) => {
                        context.achievements.write(LocalAchievementTrigger(
                            AchievementTrigger::Personal(PersonalEvent::PropPurchased {
                                price: prop.definition().price,
                            }),
                        ));
                    }
                    Err(error) => context.error.error = Some(error),
                }
            }
            Self::BuyFrame(frame) => {
                let listing = ShopAvatarFrame::definition(*frame);
                context.confirmation.show(
                    "确认购买".to_owned(),
                    format!(
                        "花费 {} 金币购买{}？\n购买后永久拥有，可随时选用。",
                        listing.price, listing.name
                    ),
                    UiAction::Shop(Self::ConfirmFramePurchase(*frame)),
                );
            }
            Self::ConfirmFramePurchase(frame) => {
                context.ui.dirty = true;
                context.error.error = context.economy.buy_frame(*frame).err();
            }
            Self::EquipFrame(frame) => {
                context.ui.dirty = true;
                match context.economy.equip_frame(*frame) {
                    Ok(changed) => {
                        context.error.error = None;
                        if changed {
                            context.achievements.write(LocalAchievementTrigger(
                                AchievementTrigger::Personal(PersonalEvent::AvatarFrameChanged),
                            ));
                        }
                    }
                    Err(error) => context.error.error = Some(error),
                }
            }
        }
    }
}
