use super::definition::personal;
use super::facts::{personal, purchased};
use crate::{AchievementDefinition, PersonalEvent};

pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] = &[
    personal!(
        "first_purchase",
        "偷打小抄",
        Bronze,
        "购买一次道具",
        purchased,
        1
    ),
    personal!(
        "expensive_purchase",
        "财大气粗",
        Bronze,
        "购买一个价格不低于200金币的道具",
        |event| personal(event, |event| matches!(
            event,
            PersonalEvent::ItemPurchased { price } if *price >= 200
        )),
        1
    ),
    personal!(
        "sakura_tunnel",
        "故乡的樱花开了",
        Bronze,
        "成功打开一次樱花联机隧道",
        |event| personal(event, |event| matches!(
            event,
            PersonalEvent::SakuraTunnelStarted
        )),
        1
    ),
    personal!(
        "ten_thousand_coins",
        "月入过万",
        Silver,
        "持有不少于10000金币",
        |event| personal(event, |event| matches!(
            event,
            PersonalEvent::CoinBalance { coins } if *coins >= 10_000
        )),
        1
    ),
    personal!(
        "hundred_purchases",
        "作弊惯犯",
        Silver,
        "购买至少100次道具",
        purchased,
        100
    ),
    personal!(
        "hundred_thousand_coins",
        "A6资产",
        Gold,
        "持有不少于100000金币",
        |event| personal(event, |event| matches!(
            event,
            PersonalEvent::CoinBalance { coins } if *coins >= 100_000
        )),
        1
    ),
    personal!(
        "thousand_purchases",
        "法外狂徒",
        Gold,
        "购买至少1000次道具",
        purchased,
        1000
    ),
];
