use super::facts::action;
use crate::McGuessEntryKind;
use leocard_achievements::{AchievementDefinition, award};

pub const DEFINITIONS: &[AchievementDefinition] = &[
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "assisted_answer",
        "又要到饭了",
        Bronze,
        "在他人给出全绿但未猜中的猜测后，自己猜出该词条答案",
        |e| action(e, |s| s.won_after_others_matching),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "wrong_length",
        "会数数吗？",
        Bronze,
        "给出与提示字数不符的猜测",
        |e| action(e, |s| s.wrong_length),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "fifteen_greens",
        "运气不能这么差吧",
        Silver,
        "同一词条累计十五次全绿未中，且其中包含自己的猜测",
        |e| action(e, |s| s.unlucky_matching),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "two_thousand_blocks",
        "像素世界",
        Silver,
        "累计猜出至少两千个方块词条",
        |e| action(e, |s| s.correct_entry.as_ref().is_some_and(|entry| {
            matches!(entry.kind, McGuessEntryKind::Block)
        })),
        2000,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "fifteen_hundred_items",
        "禁忌之物",
        Silver,
        "累计猜出至少一千五百个物品词条",
        |e| action(e, |s| s.correct_entry.as_ref().is_some_and(|entry| {
            matches!(entry.kind, McGuessEntryKind::Item)
        })),
        1500,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "five_hundred_world",
        "上帝之手",
        Silver,
        "累计猜出世界景观与生物群系词条至少五百次",
        |e| action(e, |s| s.correct_entry.as_ref().is_some_and(|entry| {
            matches!(
                entry.kind,
                McGuessEntryKind::Landscape | McGuessEntryKind::Biome
            )
        })),
        500,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "five_hundred_magic",
        "暗黑魔法",
        Silver,
        "累计猜出附魔与状态效果词条至少五百次",
        |e| action(e, |s| s.correct_entry.as_ref().is_some_and(|entry| {
            matches!(
                entry.kind,
                McGuessEntryKind::Enchantment | McGuessEntryKind::Effect
            )
        })),
        500,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "two_hundred_professions",
        "村民克星",
        Silver,
        "累计猜出至少两百个村民职业词条",
        |e| action(e, |s| s.correct_entry.as_ref().is_some_and(|entry| {
            matches!(entry.kind, McGuessEntryKind::VillagerProfession)
        })),
        200,
        Lifetime
    ),
];
