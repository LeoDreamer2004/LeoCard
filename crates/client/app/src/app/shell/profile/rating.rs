//! 玩家公共等级与参考分显示。

const REFERENCE_LEVEL_NAMES: [&str; 10] = [
    "下界合金",
    "钻石",
    "金",
    "红石",
    "铁",
    "铜",
    "圆石",
    "木头",
    "泥土",
    "堆肥桶",
];

pub(crate) fn reference_level_index(points: i32) -> usize {
    if points > 1_000 {
        0
    } else if points >= 500 {
        1
    } else if points >= 200 {
        2
    } else if points >= 100 {
        3
    } else if points >= 50 {
        4
    } else if points >= 10 {
        5
    } else if points >= 0 {
        6
    } else if points >= -10 {
        7
    } else if points >= -50 {
        8
    } else {
        9
    }
}

pub(crate) fn reference_level(points: i32) -> &'static str {
    REFERENCE_LEVEL_NAMES[reference_level_index(points)]
}
