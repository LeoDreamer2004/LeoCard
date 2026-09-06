//! 玩家公共等级与参考分显示。

pub fn reference_level(points: i32) -> &'static str {
    if points > 1_000 {
        "下界合金"
    } else if points >= 500 {
        "钻石"
    } else if points >= 200 {
        "金"
    } else if points >= 100 {
        "红石"
    } else if points >= 50 {
        "铁"
    } else if points >= 10 {
        "铜"
    } else if points >= 0 {
        "圆石"
    } else if points >= -10 {
        "木头"
    } else if points >= -50 {
        "泥土"
    } else {
        "堆肥桶"
    }
}

pub fn reference_points_label(points: i32) -> String {
    format!("等级:{}  分数:{}", reference_level(points), points)
}
