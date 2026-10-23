use leocard_protocol::GameKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ItemId {
    ObservationLens,
    ShengjiCardCounter,
    ShengjiMissingSuitCard,
    QiGui523CardCounter,
    UnoJumpInDevice,
    MahjongFanCalculator,
}

pub struct ShopItem {
    pub id: ItemId,
    pub game: GameKind,
    pub name: &'static str,
    pub description: &'static str,
    pub price: u32,
    pub duration_seconds: u64,
}

impl ItemId {
    pub const ALL: [Self; 6] = [
        Self::ObservationLens,
        Self::ShengjiCardCounter,
        Self::ShengjiMissingSuitCard,
        Self::QiGui523CardCounter,
        Self::UnoJumpInDevice,
        Self::MahjongFanCalculator,
    ];

    pub const fn definition(self) -> ShopItem {
        match self {
            Self::MahjongFanCalculator => ShopItem {
                id: self,
                game: GameKind::Mahjong,
                name: "算番卡",
                description: "在听牌提示中显示当前牌型自摸与和牌的番数。\n不足八番也会显示，花牌不计入八番起和门槛。",
                price: 60,
                duration_seconds: 3600,
            },
            Self::UnoJumpInDevice => ShopItem {
                id: self,
                game: GameKind::Uno,
                name: "抢出器",
                description: "开启后，在符合抢出规则时自动出牌。\n抢出结果受网络延迟影响，有效期内可随时开关。",
                price: 300,
                duration_seconds: 3600,
            },
            Self::QiGui523CardCounter => ShopItem {
                id: self,
                game: GameKind::QiGui523,
                name: "记牌器",
                description: "按黑、红、梅、方记录除自己手牌和已出牌外的剩余牌，标亮分牌数量。",
                price: 120,
                duration_seconds: 3600,
            },
            Self::ObservationLens => ShopItem {
                id: self,
                game: GameKind::TexasHoldem,
                name: "观局镜",
                description: "弃牌后，查看仍在对局的玩家胜率。\n仅在弃牌所在下注轮结束后可用，不显示其他玩家的底牌。",
                price: 40,
                duration_seconds: 3600,
            },
            Self::ShengjiMissingSuitCard => ShopItem {
                id: self,
                game: GameKind::Shengji,
                name: "缺门卡",
                description: "根据公开跟牌，标记其他玩家已经缺少的副牌花色或主牌。",
                price: 60,
                duration_seconds: 3600,
            },
            Self::ShengjiCardCounter => ShopItem {
                id: self,
                game: GameKind::Shengji,
                name: "记牌器",
                description: "记录除自己手牌、已出牌和自己最终埋底以外的剩余牌。",
                price: 150,
                duration_seconds: 3600,
            },
        }
    }
}
