use super::facts::{
    CONSISTENT_STREAK, ENTRY_COLLECTION, VERSION_COLLECTION, action, collected, hand,
};
use leocard_achievements::{AchievementDefinition, AchievementTrigger, award};

pub const DEFINITIONS: &[AchievementDefinition] = &[
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "first_win",
        "旗开得胜",
        Bronze,
        "猜中答案，获得第一局游戏胜利",
        |e| action(e, |s| s.correct_entry.is_some()),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "matching_wrong",
        "MIT 协议",
        Bronze,
        "在某个词条中首次给出全绿但没有猜中的猜测",
        |e| action(e, |s| s.first_matching_wrong),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "unknown",
        "云玩鉴定",
        Bronze,
        "猜出一个不存在的词条",
        |e| action(e, |s| s.unknown),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "skip_then_win",
        "静观其变",
        Bronze,
        "跳过一次猜测，随后在本局的之后回合中猜出正确答案",
        |e| action(e, |s| s.won_after_skip),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "three_unknowns",
        "打回家重玩",
        Bronze,
        "一局中自己连续猜出三个不存在的词条",
        |e| action(e, |s| s.unknown_run >= 3),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "three_skips",
        "不想玩就直说",
        Bronze,
        "一局中自己连续跳过三次猜测",
        |e| action(e, |s| s.skip_run >= 3),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "all_wrong",
        "真的玩过 MC 吗",
        Bronze,
        "除第一个猜测外，给出一个所有特征全错的答案",
        |e| action(e, |s| s.all_wrong_after_first),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "end_matching_run",
        "我是来结束这一切的",
        Bronze,
        "至少连续五个错误猜测的所有特征均匹配后，猜出正确答案",
        |e| action(e, |s| s.ended_matching_run),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "first_cycle",
        "速战速决",
        Bronze,
        "第一圈中猜出正确答案",
        |e| action(e, |s| s.first_cycle_win),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "opening_win",
        "忘关就是没开",
        Silver,
        "猜词模式中作为本局第一个猜测玩家，直接猜出答案",
        |e| action(e, |s| s.opening_win),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "others_skip",
        "你们还得练",
        Silver,
        "至少三人时，其他玩家全部跳过后猜出正确答案",
        |e| action(e, |s| s.opponents_all_skipped),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "round_sweep",
        "史蒂夫本人",
        Silver,
        "至少三人时，赢下某一轮内的全部小局",
        |e| hand(e, |s| s.round_sweep),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "twelve_guesses",
        "鹿死谁手",
        Silver,
        "一局总共猜测至少十二次后，猜出正确答案",
        |e| action(e, |s| s.long_search_win),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "all_versions",
        "版本横行者",
        Silver,
        "累计在词条涵盖的每个 Minecraft 游戏版本中都猜出至少一个词条",
        |e| collected(e, VERSION_COLLECTION),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "ten_thousand",
        "骨灰级玩家",
        Gold,
        "累计猜中一万个词条",
        |e| action(e, |s| s.correct_entry.is_some()),
        10000,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "all_entries",
        "倒背如流",
        Gold,
        "累计猜中全部现存方块与物品词条",
        |e| collected(e, ENTRY_COLLECTION),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "match_sweep",
        "人肉数据库",
        Gold,
        "猜词中至少三人且配置至少三轮，赢下整场的全部小局",
        |e| hand(e, |s| s.match_sweep),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "consistent_five",
        "稳健型选手",
        Gold,
        "连续五个小局中，自己的每次有效猜测匹配度都不低于上一个猜测；每局至少猜测一次且不猜不存在的词条",
        |e| match e {
            AchievementTrigger::Streak { id, qualifies } if *id == CONSISTENT_STREAK =>
                u64::from(*qualifies),
            _ => 0,
        },
        5,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        MinecraftGuess,
        "no_answers_top",
        "没猜中也能赢？",
        Gold,
        "整场没有猜中答案，各小局均有人猜中，仍取得本场最高分",
        |e| hand(e, |s| s.no_answers_top),
        1,
        Lifetime
    ),
];
