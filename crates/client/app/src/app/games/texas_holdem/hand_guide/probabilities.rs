use leocard_texas_holdem::{TexasHoldemHandCategory, TexasHoldemRuleSet};

/// 五张公共牌全部发出后，一名随机玩家的最终最佳牌型分布。
/// 不以底牌、下注或未弃牌为条件；忽略踢脚牌不改变牌型分布。
pub(super) struct HandGuideProbabilities {
    counts: [u64; 10],
    total: u64,
}

impl HandGuideProbabilities {
    pub(super) fn for_rules(rules: &TexasHoldemRuleSet) -> &'static Self {
        match (rules.short_deck, rules.omaha) {
            (false, false) => &STANDARD_HOLDEM,
            (true, false) => &SHORT_HOLDEM,
            (false, true) => &STANDARD_OMAHA,
            (true, true) => &SHORT_OMAHA,
        }
    }

    pub(super) fn percentage_label(&self, category: TexasHoldemHandCategory) -> String {
        let percentage = self.percentage(category);
        let precision = if percentage > 0.0 && percentage < 0.01 {
            // 小概率保留两位有效数字，避免非零概率被舍入为零。
            (1.0 - percentage.log10().floor()) as usize
        } else {
            2
        };
        format!("{percentage:.precision$}%")
    }

    fn percentage(&self, category: TexasHoldemHandCategory) -> f64 {
        // 显式映射，避免依赖游戏枚举的声明顺序。
        let index = match category {
            TexasHoldemHandCategory::HighCard => 0,
            TexasHoldemHandCategory::OnePair => 1,
            TexasHoldemHandCategory::TwoPair => 2,
            TexasHoldemHandCategory::Straight => 3,
            TexasHoldemHandCategory::ThreeOfAKind => 4,
            TexasHoldemHandCategory::Flush => 5,
            TexasHoldemHandCategory::FullHouse => 6,
            TexasHoldemHandCategory::FourOfAKind => 7,
            TexasHoldemHandCategory::StraightFlush => 8,
            TexasHoldemHandCategory::RoyalFlush => 9,
        };
        self.counts[index] as f64 * 100.0 / self.total as f64
    }
}

// 精确组合计数，皇家同花顺与普通同花顺分别计数，每副牌只归入最高牌型。
// 德州枚举 C(N, 7) 副牌；短牌按同花 > 葫芦、三条 > 顺子归类。
const STANDARD_HOLDEM: HandGuideProbabilities = HandGuideProbabilities {
    counts: [
        23_294_460, 58_627_800, 31_433_400, 6_180_020, 6_461_620, 4_047_644, 3_473_184, 224_848,
        37_260, 4_324,
    ],
    total: 133_784_560,
};

const SHORT_HOLDEM: HandGuideProbabilities = HandGuideProbabilities {
    counts: [
        233_100, 2_316_600, 3_157_056, 1_139_580, 637_560, 175_560, 633_024, 44_640, 8_700, 1_860,
    ],
    total: 8_347_680,
};

// 奥马哈的样本空间为 C(N, 4) * C(N - 4, 5)，严格使用两张底牌加三张公共牌。
// 按底牌/公共牌的点数重数枚举，以各点数的花色选择数加权；
// 固定一种花色计数同花后乘四，以容斥扣除重叠的同花顺，再单独提取皇家同花顺。
const STANDARD_OMAHA: HandGuideProbabilities = HandGuideProbabilities {
    counts: [
        13_851_662_832,
        122_655_542_152,
        170_775_844_104,
        52_289_648_688,
        40_712_657_408,
        31_216_782_384,
        29_424_798_576,
        2_225_270_496,
        368_486_160,
        42_807_600,
    ],
    total: 463_563_500_400,
};

const SHORT_OMAHA: HandGuideProbabilities = HandGuideProbabilities {
    counts: [
        0,
        512_953_992,
        3_929_816_376,
        3_001_418_496,
        1_646_821_008,
        594_683_640,
        1_952_576_640,
        180_989_568,
        35_241_960,
        7_551_600,
    ],
    total: 11_862_053_280,
};
