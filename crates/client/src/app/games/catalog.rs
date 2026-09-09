//! 客户端支持的游戏目录与大厅摘要。

use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::{GameKind, GameRules, TABLE_SEAT_COUNT};
use leocard_shengji::ShengjiRuleSet;
use leocard_uno::UnoRuleSet;

#[derive(Clone, Copy)]
pub(crate) struct GameDescriptor {
    pub kind: GameKind,
    pub title: &'static str,
    pub description: &'static str,
}

pub(crate) const SUPPORTED_GAMES: [GameDescriptor; 5] = [
    GameDescriptor {
        kind: GameKind::QiGui523,
        title: "七鬼五二三",
        description: "放空大脑, 有牌就出",
    },
    GameDescriptor {
        kind: GameKind::TexasHoldem,
        title: "德州扑克",
        description: "窝要验牌!",
    },
    GameDescriptor {
        kind: GameKind::Shengji,
        title: "升级",
        description: "神对手 or 猪队友",
    },
    GameDescriptor {
        kind: GameKind::Uno,
        title: "UNO",
        description: "最后一张，记得喊 UNO!",
    },
    GameDescriptor {
        kind: GameKind::Mahjong,
        title: "麻将合集",
        description: "八番起和，方城之战",
    },
];

pub(crate) fn game_seat_count(rules: &GameRules) -> u8 {
    match rules {
        GameRules::Shengji(_) => ShengjiRuleSet::PLAYER_COUNT as u8,
        GameRules::Uno(_) => UnoRuleSet::MAX_PLAYERS,
        GameRules::Mahjong(_) => MahjongRuleSet::PLAYER_COUNT as u8,
        GameRules::QiGui523(_) | GameRules::TexasHoldem(_) => TABLE_SEAT_COUNT,
    }
}

pub(crate) fn lobby_rule_labels(rules: &GameRules) -> [String; 3] {
    match rules {
        GameRules::QiGui523(rules) => [
            format!("{}副", rules.deck_count),
            format!("{}张", rules.hand_size),
            crate::app::time_control_label(rules.time_control).to_owned(),
        ],
        GameRules::TexasHoldem(rules) => [
            format!("{}筹码", rules.starting_chips),
            match (rules.omaha, rules.short_deck) {
                (true, true) => "短牌奥马哈".to_owned(),
                (true, false) => "奥马哈".to_owned(),
                (false, true) => "短牌德州".to_owned(),
                (false, false) => "标准德州".to_owned(),
            },
            if rules.ignore_kickers {
                "只比较最大牌型".to_owned()
            } else {
                "标准比牌".to_owned()
            },
        ],
        GameRules::Shengji(rules) => [
            format!("{}副牌", rules.deck_count),
            if rules.bottom_copy {
                "允许抄底".to_owned()
            } else {
                "不抄底".to_owned()
            },
            if rules.five_trump_crossing {
                "五主过江".to_owned()
            } else {
                "不过江".to_owned()
            },
        ],
        GameRules::Uno(rules) if rules.is_no_mercy() => [
            "No Mercy".to_owned(),
            "+2 至 +10 递增堆叠".to_owned(),
            if rules.no_mercy.mercy_elimination {
                "25 张淘汰".to_owned()
            } else {
                "不启用慈悲淘汰".to_owned()
            },
        ],
        GameRules::Uno(rules) if rules.is_flip() => [
            "UNO FLIP".to_owned(),
            if rules.flip.random_pairing {
                "随机双面配对".to_owned()
            } else {
                "固定双面配对".to_owned()
            },
            if rules.flip.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
        ],
        GameRules::Uno(rules) => [
            if rules.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
            if rules.jump_in {
                "允许抢出".to_owned()
            } else {
                "不抢出".to_owned()
            },
            if rules.uno_callout {
                "UNO 检举".to_owned()
            } else {
                "不检举".to_owned()
            },
        ],
        GameRules::Mahjong(rules) => [
            match rules.match_length {
                MahjongMatchLength::SingleHand => "单局结算".to_owned(),
                MahjongMatchLength::EastRound => "东风场".to_owned(),
                MahjongMatchLength::HalfGame => "半庄场".to_owned(),
                MahjongMatchLength::FullGame => "全庄场".to_owned(),
            },
            if rules.minimum_eight_points {
                "8 番起和".to_owned()
            } else {
                "不限起和番数".to_owned()
            },
            if rules.multiple_winners {
                "允许一炮多响".to_owned()
            } else {
                "截和".to_owned()
            },
        ],
    }
}
