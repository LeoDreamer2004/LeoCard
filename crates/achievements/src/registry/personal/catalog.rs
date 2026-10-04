use super::facts::*;
use crate::{
    PersonalEvent,
    registry::{macros::award, types::*},
};
use leocard_protocol::PlayerInteractionKind;
use std::time::Duration;

macro_rules! personal {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr) => {
        award!(
            "leocard:personal/",
            Personal,
            $id,
            $title,
            $tier,
            $description,
            $amount,
            $target,
            Lifetime
        )
    };
}

pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] = &[
    personal!(
        "avatar",
        "改头换面",
        Bronze,
        "为自己制作一个头像",
        |event| personal(event, |event| matches!(event, PersonalEvent::AvatarSaved)),
        1
    ),
    personal!(
        "update",
        "追踪实事",
        Bronze,
        "通过游戏内更新进行一次版本升级",
        |event| personal(event, |event| matches!(event, PersonalEvent::ClientUpdated)),
        1
    ),
    personal!(
        "background",
        "牌桌见闻",
        Bronze,
        "更换一次游戏内的牌桌背景",
        |event| personal(event, |event| matches!(
            event,
            PersonalEvent::TableBackgroundChanged
        )),
        1
    ),
    personal!(
        "first_egg",
        "一点小小的失误",
        Bronze,
        "获得一个鸡蛋",
        |event| received(event, PlayerInteractionKind::Egg),
        1
    ),
    personal!(
        "wine",
        "我打的还不错吧",
        Bronze,
        "收获一个酒杯",
        |event| received(event, PlayerInteractionKind::Wine),
        1
    ),
    personal!(
        "flowers_sent",
        "赠人玫瑰，手有余香",
        Bronze,
        "送出100朵花",
        flowers_sent,
        100
    ),
    personal!(
        "first_voice",
        "我要讲一句",
        Bronze,
        "发送一条快捷语音",
        quick_voice,
        1
    ),
    personal!(
        "five_shoes",
        "怎么这么菜啊",
        Bronze,
        "一分钟内连续向同一名玩家扔出五个拖鞋",
        |event| personal(
            event,
            |event| matches!(event, PersonalEvent::InteractionRun { kind: PlayerInteractionKind::Shoe, count, span } if *count >= 5 && *span <= Duration::from_secs(60))
        ),
        1
    ),
    personal!(
        "first_host",
        "你们都得听我的",
        Bronze,
        "作为房主开始一场游戏",
        hosted,
        1
    ),
    personal!(
        "thousand_voices",
        "请勿喧哗",
        Silver,
        "发送1000条快捷语音",
        quick_voice,
        1000
    ),
    personal!(
        "text_chat",
        "这里不需要键盘手",
        Silver,
        "在游戏内打字聊天总计超过1000字符",
        text_characters,
        1001
    ),
    personal!(
        "ten_thousand_flowers",
        "美名远扬",
        Silver,
        "获得至少10000个鲜花",
        |event| received(event, PlayerInteractionKind::Flower),
        10_000
    ),
    personal!(
        "ten_thousand_eggs",
        "臭名昭著",
        Silver,
        "获得至少10000个鸡蛋",
        |event| received(event, PlayerInteractionKind::Egg),
        10_000
    ),
    personal!(
        "five_hundred_host",
        "在下正是吕伯奢",
        Silver,
        "作为房主开始500场游戏",
        hosted,
        500
    ),
    personal!(
        "ten_thousand_voices",
        "顶级话痨",
        Gold,
        "发送10000条快捷语音",
        quick_voice,
        10_000
    ),
    personal!(
        "hundred_thousand_flowers",
        "流芳百世",
        Gold,
        "获得至少100000个鲜花",
        |event| received(event, PlayerInteractionKind::Flower),
        100_000
    ),
    personal!(
        "hundred_thousand_eggs",
        "遗臭万年",
        Gold,
        "获得至少100000个鸡蛋",
        |event| received(event, PlayerInteractionKind::Egg),
        100_000
    ),
    personal!(
        "five_gold",
        "牢玩家的资历",
        Gold,
        "获得至少5个金色奖杯",
        |event| u64::from(
            matches!(event, AchievementTrigger::TrophyTotals(counts) if counts.gold >= 5)
        ),
        1
    ),
];
