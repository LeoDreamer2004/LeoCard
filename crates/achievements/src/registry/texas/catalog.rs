use super::super::{macros::award, types::*};
use super::facts::*;
use leocard_texas_holdem::{
    TexasHoldemHandCategory as Category, TexasHoldemRank as Rank, TexasHoldemRuleSet,
    TexasHoldemStreet,
};

macro_rules! texas {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr) => {
        award!(
            "leocard:texas/",
            TexasHoldem,
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

macro_rules! category {
    ($id:literal, $title:literal, $tier:ident, $category:ident, $description:literal) => {
        texas!(
            $id,
            $title,
            $tier,
            $description,
            |event| hand(event, |facts| facts.won_chips > 0
                && (facts.category == Some(Category::$category)
                    || (Category::$category == Category::StraightFlush
                        && facts.category == Some(Category::RoyalFlush)))),
            1
        )
    };
}

pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] = &[
    texas!("first_pot", "开张第一手", Bronze, "第一次赢下底池", wins, 1),
    texas!("ten_wins", "十手小胜", Bronze, "累计赢下10手牌", wins, 10),
    category!(
        "high_card",
        "高牌也能赢啊？",
        Bronze,
        HighCard,
        "使用「高牌」赢下一手"
    ),
    category!(
        "one_pair",
        "平平无奇",
        Bronze,
        OnePair,
        "使用「对子」赢下一手"
    ),
    category!(
        "two_pair",
        "两对也不差",
        Bronze,
        TwoPair,
        "使用「两对」赢下一手"
    ),
    category!(
        "three_of_a_kind",
        "三张同号",
        Bronze,
        ThreeOfAKind,
        "使用「三条」赢下一手"
    ),
    category!(
        "straight",
        "顺流而下",
        Bronze,
        Straight,
        "使用「顺子」赢下一手"
    ),
    category!("flush", "就是要买花", Bronze, Flush, "使用「同花」赢下一手"),
    category!(
        "full_house",
        "不行，还不能笑",
        Bronze,
        FullHouse,
        "使用「葫芦」赢下一手"
    ),
    category!(
        "four_of_a_kind",
        "重磅炸弹",
        Bronze,
        FourOfAKind,
        "使用「四条」赢下一手"
    ),
    texas!(
        "large_all_in",
        "没有梭哈玩什么游戏",
        Bronze,
        "一次全下本轮下注总额不少于20BB",
        |event| action(event, |facts| facts.all_in_amount
            >= 20 * TexasHoldemRuleSet::BIG_BLIND),
        1
    ),
    texas!(
        "three_bet",
        "加注上瘾",
        Bronze,
        "在一手中做一次翻前3-bet",
        |event| action(event, |facts| facts.street == TexasHoldemStreet::PreFlop
            && facts.full_raise
            && facts.bet_level == 3),
        1
    ),
    texas!(
        "frequent_raiser",
        "朋友公敌",
        Bronze,
        "完成至少5手的一场游戏，至少一半手牌有主动加注",
        |event| hand(event, |facts| facts.match_complete
            && facts.match_statistics.hands >= 5
            && u64::from(facts.match_statistics.raised_hands) * 2
                >= u64::from(facts.match_statistics.hands)),
        1
    ),
    texas!(
        "button_win",
        "庄家优势",
        Bronze,
        "在按钮位赢下底池",
        |event| hand(event, |facts| facts.won_chips > 0 && facts.button),
        1
    ),
    texas!(
        "blind_defense",
        "守盲有功",
        Bronze,
        "在大盲位跟注或反加翻前加注，并赢下底池",
        |event| hand(event, |facts| facts.won_chips > 0
            && facts.big_blind
            && facts.defended_blind),
        1
    ),
    texas!(
        "ten_folds",
        "标准怂包",
        Bronze,
        "在同一场游戏中连续弃牌10手",
        |event| hand(event, |facts| facts.match_statistics.consecutive_folds
            >= 10),
        1
    ),
    texas!(
        "uncontested",
        "无人应战",
        Bronze,
        "其他玩家全部弃牌，直接赢下底池",
        |event| hand(event, |facts| facts.won_chips > 0 && !facts.showdown),
        1
    ),
    texas!(
        "half_chips",
        "小有盈余",
        Bronze,
        "一场游戏结束时获得全桌至少一半筹码",
        |event| hand(event, |facts| facts.match_complete
            && u64::from(facts.final_stack) * 2 >= u64::from(facts.total_chips)),
        1
    ),
    texas!(
        "premium_three_bet",
        "好牌慢打",
        Bronze,
        "持AA或KK做翻前3-bet，并赢下该手",
        |event| hand(event, |facts| facts.won_chips > 0
            && facts.preflop_bet_levels.contains(&3)
            && (pair(facts, Rank::Ace) || pair(facts, Rank::King))),
        1
    ),
    texas!(
        "five_hundred_wins",
        "五百胜场",
        Silver,
        "累计赢下500手牌",
        wins,
        500
    ),
    texas!(
        "aces_vs_pair",
        "冤家路窄",
        Silver,
        "持AA在摊牌中击败另一家的口袋对子",
        |event| hand(event, |facts| facts.won_chips > 0
            && pair(facts, Rank::Ace)
            && facts.defeated_pocket_pairs >= 1),
        1
    ),
    texas!(
        "river_value",
        "河牌收租",
        Silver,
        "河牌下注或加注被跟注，并在摊牌中赢下底池",
        |event| hand(event, |facts| facts.won_chips > 0
            && facts.showdown
            && facts.river_bet_called),
        1
    ),
    texas!(
        "short_stack_comeback",
        "短码回春",
        Silver,
        "曾以不足5BB开始一手，最终获得整场第一",
        |event| hand(event, |facts| facts.match_complete
            && facts.first_place
            && facts.match_statistics.lowest_starting_stack
                < 5 * TexasHoldemRuleSet::BIG_BLIND),
        1
    ),
    texas!(
        "three_quarter_chips",
        "友尽局",
        Silver,
        "一场游戏结束时获得全桌至少四分之三筹码",
        |event| hand(event, |facts| facts.match_complete
            && u64::from(facts.final_stack) * 4
                >= u64::from(facts.total_chips) * 3),
        1
    ),
    texas!(
        "three_different_wins",
        "三连不同",
        Silver,
        "在同一场游戏中连续3手用不同牌型赢下",
        |event| hand(event, |facts| facts
            .match_statistics
            .consecutive_winning_categories
            .len()
            >= 3),
        1
    ),
    category!(
        "straight_flush",
        "我还想要两个字的前缀！",
        Silver,
        StraightFlush,
        "使用「同花顺」赢下一手"
    ),
    texas!(
        "offsuit_two_seven",
        "垃圾牌的奇迹",
        Silver,
        "持不同花色的2和7赢下一手",
        |event| hand(event, |facts| facts.won_chips > 0
            && offsuit_two_seven(facts)),
        1
    ),
    texas!(
        "side_pot_raise",
        "贪婪到底",
        Silver,
        "至少两家已全下时在边池加注，并赢下边池",
        |event| hand(event, |facts| facts.won_side_pot_chips > 0
            && facts.side_pot_raises > 0),
        1
    ),
    category!(
        "royal_flush",
        "皇家降临",
        Gold,
        RoyalFlush,
        "使用「皇家同花顺」赢下一手"
    ),
    texas!(
        "five_thousand_wins",
        "澳门赌神",
        Gold,
        "累计赢下5000手牌",
        wins,
        5000
    ),
    texas!(
        "straight_flush_vs_quads",
        "龙争虎斗",
        Gold,
        "在同一底池中用同花顺击败四条",
        |event| hand(event, |facts| facts.won_chips > 0
            && matches!(
                facts.category,
                Some(Category::StraightFlush | Category::RoyalFlush)
            )
            && facts.defeated_categories.contains(&Category::FourOfAKind)),
        1
    ),
    texas!(
        "three_pocket_pairs",
        "三家冤家",
        Gold,
        "持AA与另两家的口袋对子翻前全下，并在摊牌中击败双方",
        |event| hand(event, |facts| facts.won_chips > 0
            && pair(facts, Rank::Ace)
            && facts.preflop_all_in
            && facts.defeated_preflop_all_in_pairs >= 2),
        1
    ),
    texas!(
        "all_chips",
        "判你赢得了",
        Gold,
        "一场游戏结束时获得全桌所有筹码",
        |event| hand(event, |facts| facts.match_complete
            && facts.total_chips > 0
            && facts.final_stack == facts.total_chips),
        1
    ),
    texas!(
        "river_bluff",
        "空城计",
        Gold,
        "用高牌河牌全下赢下底池，使对手弃掉三条或更强的牌",
        |event| hand(event, |facts| facts.won_chips > 0
            && !facts.showdown
            && facts.category == Some(Category::HighCard)
            && facts
                .river_all_in_folded_categories
                .iter()
                .any(|category| at_least_trips(*category, facts.short_deck))),
        1
    ),
    texas!(
        "utg_two_seven",
        "德州玩家的荣耀",
        Gold,
        "UTG位持不同花色的2和7，赢下不少于20BB的底池",
        |event| hand(event, |facts| facts.won_chips > 0
            && facts.under_the_gun
            && offsuit_two_seven(facts)
            && facts.won_pot >= 20 * TexasHoldemRuleSet::BIG_BLIND),
        1
    ),
    texas!(
        "perfect_match",
        "你是怎么做到的？",
        Gold,
        "完成一场至少8手的游戏，并赢下其中每一手",
        |event| hand(event, |facts| facts.match_complete
            && facts.match_statistics.hands >= 8
            && facts.match_statistics.won_hands == facts.match_statistics.hands),
        1
    ),
];
